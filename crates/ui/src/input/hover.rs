//! What the pointer is over, and the highlight drawn for it: a message row,
//! a folder, an account, a key bar button, a clickable footer item, a divider.
//! The highlight is only a colour on the cells under the thing, drawn after
//! everything else, so no widget has to know about it. Nothing is lit while a
//! screen or a question is open over the mail, while something is being
//! dragged, or after the window lost focus. While rows are being dragged the
//! folder a drop would move them to is lit instead. A link in the reader is
//! lit too, but by the reader's own drawing (`link_light`), under the marks:
//! here it is only a thing the pointer can be over, so that moving between
//! links, or off one, asks for a new drawing.

use surface::{Color, Frame, InputEvent, Modifier, Rect};

use super::drag::{Drag, divider_at, divider_rect};
use super::hit::Hit;
use super::motion::folders_height;
use crate::app_view::AppView;
use crate::layout::{Layout, SectionKind};
use crate::theme::roles;
use crate::widgets::{DrawContext, FooterSpan, button_span_at, row_offsets, visible_rows};

/// The thing the pointer is over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverTarget {
    Folder(usize),
    Message(usize),
    Account(usize),
    /// A key bar button: its action, where it starts and how wide it is.
    KeyBar(&'static str, u16, u16),
    Footer(FooterSpan),
    Divider(Drag),
    /// A link of the message in the reader, by its place among the message's
    /// links: two links with one address are two targets.
    Link(usize),
}

/// Gives every cell of `rect` the background `color`, keeping its text.
fn tint(frame: &mut Frame, rect: Rect, color: Option<Color>) {
    let Some(color) = color else {
        return;
    };
    let area = frame.area().intersection(rect);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = frame.cells.cell_mut((x, y)) {
                cell.bg = color;
            }
        }
    }
}

impl AppView {
    /// Whether nothing holds the pointer: no screen or question is open over
    /// the mail and no bar, thumb, rows or text are being dragged. The press on
    /// a link that has not yet become a selection holds nothing.
    pub(crate) fn pointer_is_free(&self) -> bool {
        let selecting = self.reading.selection.as_ref().is_some_and(|held| held.dragging && held.spread);
        self.mail_is_free() && self.drag.is_none() && self.bar_drag.is_none() && !self.dragging_rows() && !selecting
    }

    /// What the pointer is over, when something there is lit by it.
    pub(crate) fn hover_target(&self, grid: (u16, u16)) -> Option<HoverTarget> {
        let (col, row) = self.memory.hover?;
        if !self.pointer_is_free() {
            return None;
        }
        let layout = self.layout(grid.0, grid.1);
        if layout.too_small {
            return None;
        }
        if let Some(drag) = divider_at(&layout, col, row) {
            return Some(HoverTarget::Divider(drag));
        }
        match self.hit(&layout, col, row) {
            Hit::Message(index) => Some(HoverTarget::Message(index)),
            Hit::Folder(index) => Some(HoverTarget::Folder(index)),
            Hit::Account(index) => Some(HoverTarget::Account(index)),
            Hit::KeyBar(_) => {
                let (action, x, width) = button_span_at(layout.key_bar?, &self.config.key_bar.buttons, &self.keys, col)?;
                Some(HoverTarget::KeyBar(action, x, width))
            }
            Hit::Nothing => self.footer_span_at(&layout, col, row).map(HoverTarget::Footer),
            // The link is looked for only when the pointer is on the reader.
            Hit::Reader if self.link_light_allowed() => self.link_index_at(&layout, col, row).map(HoverTarget::Link),
            Hit::Sidebar | Hit::ListChrome | Hit::Reader => None,
        }
    }

    /// Notes where the pointer is (or that it left with the window's focus).
    /// Returns whether what is lit changed, so the screen is drawn again.
    pub(crate) fn track_hover(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        let at = match event {
            InputEvent::PointerMoved { pos, .. } | InputEvent::Button { pos, .. } => Some((pos.col, pos.row)),
            InputEvent::Focus { gained: false } => {
                // A button let go while the window was not looking never arrives.
                self.cancel_row_drag();
                None
            }
            _ => return false,
        };
        let before = self.hover_target(grid);
        self.memory.hover = at;
        let after = self.hover_target(grid);
        if matches!(after, Some(HoverTarget::Link(_))) && before != after {
            // The pointer came onto a link: the notes now showing are older.
            self.memory.hint_note = self.footer_note().map(str::to_string);
        }
        before != after
    }

    /// Lights what the pointer is over, or the folder a drag would drop on.
    pub(crate) fn draw_hover(&self, frame: &mut Frame, ctx: &DrawContext<'_>, layout: &Layout) {
        let area = frame.area();
        if let Some(folder) = self.drag_over() {
            let rows = visible_rows(&self.mail.folders);
            let index = rows.iter().position(|row| row.folder.id == folder);
            if let Some(rect) = index.and_then(|index| self.folder_row_rect(layout, index)) {
                tint(frame, rect, ctx.bg_color(roles::SIDEBAR_HOVER));
            }
            return;
        }
        let Some(target) = self.hover_target((area.width, area.height)) else {
            return;
        };
        match target {
            HoverTarget::Message(index) if index != self.selection.message_index => {
                if let Some(rect) = self.message_row_rect(layout, index) {
                    tint(frame, rect, ctx.bg_color(roles::LIST_ROW_HOVER));
                }
            }
            HoverTarget::Folder(index) => {
                let rows = visible_rows(&self.mail.folders);
                if rows.get(index).is_some_and(|row| Some(row.folder.id) != self.selection.folder)
                    && let Some(rect) = self.folder_row_rect(layout, index)
                {
                    tint(frame, rect, ctx.bg_color(roles::SIDEBAR_HOVER));
                }
            }
            HoverTarget::Account(index) if self.account_cursor_row() != Some(index) => {
                let body = layout.sidebar.as_ref().and_then(|sidebar| sidebar.section(SectionKind::Accounts)).map(|section| section.body);
                if let Some(body) = body {
                    tint(frame, Rect::new(body.x, body.y + u16::try_from(index).unwrap_or(0), body.width, 1), ctx.bg_color(roles::SIDEBAR_HOVER));
                }
            }
            HoverTarget::KeyBar(_, x, width) => {
                if let Some(bar) = layout.key_bar {
                    tint(frame, Rect::new(x, bar.y, width, 1), ctx.bg_color(roles::KEYBAR_HOVER));
                }
            }
            HoverTarget::Footer(span) => {
                if let Some(footer) = layout.footer {
                    tint(frame, Rect::new(span.x, footer.y, span.width, 1), ctx.bg_color(roles::KEYBAR_HOVER));
                }
            }
            HoverTarget::Divider(drag) => light_divider(frame, ctx, layout, drag),
            _ => {}
        }
    }

    /// The cells of message row `index`, when it shows.
    pub(crate) fn message_row_rect(&self, layout: &Layout, index: usize) -> Option<Rect> {
        let list = layout.list.as_ref()?;
        let line = index.checked_sub(self.list_first(layout))?;
        let y = list.rows.y.checked_add(u16::try_from(line).ok()?)?;
        (y < list.rows.bottom()).then_some(Rect::new(list.rows.x, y, list.rows.width, 1))
    }

    /// The cells of folder row `index` (of the visible rows), when it shows.
    pub(crate) fn folder_row_rect(&self, layout: &Layout, index: usize) -> Option<Rect> {
        let sidebar = layout.sidebar.as_ref()?;
        let body = sidebar.section(SectionKind::Folders)?.body;
        let rows = visible_rows(&self.mail.folders);
        let offsets = row_offsets(&rows, sidebar.spacing);
        let first = self.folders_view_first(&rows, &offsets, folders_height(layout));
        let dy = offsets.get(index)?.checked_sub(*offsets.get(first)?)?;
        (dy < body.height).then_some(Rect::new(body.x, body.y + dy, body.width, 1))
    }
}

/// The divider's cells in the hover colour, bold.
fn light_divider(frame: &mut Frame, ctx: &DrawContext<'_>, layout: &Layout, drag: Drag) {
    let Some(rect) = divider_rect(layout, drag) else {
        return;
    };
    let color = ctx.fg_color(roles::BORDER_HOVER);
    let area = frame.area().intersection(rect);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = frame.cells.cell_mut((x, y)) {
                cell.fg = color;
                cell.modifier.insert(Modifier::BOLD);
            }
        }
    }
}

#[cfg(test)]
include!("hover.tests.rs");
