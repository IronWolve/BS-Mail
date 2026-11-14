//! Input events into changes of selection, focus and scroll.
//!
//! Nothing here talks to the backend. Selection moves; the commands the new
//! selection needs are worked out afterwards by `AppView::take_commands`.

use model::Command;
use surface::{ButtonState, InputEvent, PointerButton};

use super::action::{Action, action_for};
use super::compose::ComposeKind;
use super::hit::{Hit, HitView, hit_at};
use super::movement::{WHEEL_ROWS, step};
use super::viewport::{folders_first, list_first};
use crate::app_view::AppView;
use crate::layout::{Layout, SectionKind};
use crate::view_state::{Focus, ListPage, OverlayKind};
use crate::widgets::{BodyView, DrawContext, HighlightRules, row_offsets, visible_rows, visual_len};

impl AppView {
    /// Reacts to one input event on a `cols` x `rows` grid. Returns whether
    /// anything on screen may have changed, so the host knows to redraw.
    pub fn handle_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        if self.composer.is_some() {
            return self.composer_input(event, grid);
        }
        if self.overlays.is_open() {
            return self.input_over_overlay(event);
        }
        match event {
            InputEvent::Key(key) => match action_for(key) {
                Some(action) => {
                    self.act(action, grid);
                    true
                }
                None => false,
            },
            InputEvent::Button {
                button: PointerButton::Left,
                state: ButtonState::Pressed,
                pos,
                ..
            } => {
                if self.begin_drag(pos.col, pos.row, grid) {
                    return true;
                }
                self.click(pos.col, pos.row, grid)
            }
            InputEvent::Button {
                button: PointerButton::Left,
                state: ButtonState::Released,
                ..
            } => self.end_drag(),
            InputEvent::PointerMoved { pos, .. } => self.drag_to(*pos, grid),
            InputEvent::Wheel { pos, rows, .. } => self.wheel(pos.col, pos.row, *rows, grid),
            _ => false,
        }
    }

    /// While a screen is open over this one, a key press or a click closes
    /// it and nothing else reaches the screen underneath.
    fn input_over_overlay(&mut self, event: &InputEvent) -> bool {
        let closes = matches!(
            event,
            InputEvent::Key(_)
                | InputEvent::Button {
                    state: ButtonState::Pressed,
                    ..
                }
        );
        if closes {
            self.overlays.open = None;
        }
        closes
    }

    /// The commands the backend is due, in order. Call after every input
    /// event and every backend event.
    pub fn take_commands(&mut self) -> Vec<Command> {
        self.queue.take(&self.selection, &self.mail)
    }

    /// Asks the backend to look at the server again, for the open folder and
    /// message too.
    pub fn refresh(&mut self) {
        self.queue.forget();
        if let Some(account) = self.selection.account.clone() {
            self.queue.push(Command::SyncNow { account });
        }
    }

    fn act(&mut self, action: Action, grid: (u16, u16)) {
        match action {
            Action::FocusNext => self.focus = self.focus.next(),
            Action::FocusPrevious => self.focus = self.focus.previous(),
            Action::Enter => self.focus = deeper(self.focus),
            Action::Back => self.focus = shallower(self.focus),
            Action::Right if self.focus == Focus::Sidebar => self.open_folder_or_go_in(),
            Action::Right => self.focus = deeper(self.focus),
            Action::Left if self.focus == Focus::Sidebar => self.close_folder_or_go_up(),
            Action::Left => self.focus = shallower(self.focus),
            Action::SpaceDown if self.focus == Focus::Reader => {
                self.act(Action::PageDown, grid);
            }
            Action::SpaceUp if self.focus == Focus::Reader => self.act(Action::PageUp, grid),
            Action::SpaceDown | Action::SpaceUp => {}
            Action::ToggleRead => self.toggle_read(),
            Action::ToggleStar => self.toggle_star(),
            Action::Delete => self.delete_selected(),
            Action::Sync => self.refresh(),
            Action::Help => self.overlays.open = Some(OverlayKind::Help),
            Action::Compose => self.start_compose(ComposeKind::New),
            Action::Reply => self.start_compose(ComposeKind::Reply),
            Action::ReplyAll => self.start_compose(ComposeKind::ReplyAll),
            Action::Forward => self.start_compose(ComposeKind::Forward),
            movement => {
                let layout = self.layout(grid.0, grid.1);
                match self.focus {
                    Focus::Sidebar => self.move_folder(movement, &layout),
                    Focus::List => self.move_message(movement, &layout),
                    Focus::Reader => self.scroll_reader(movement, &layout),
                }
            }
        }
    }

    fn click(&mut self, col: u16, row: u16, grid: (u16, u16)) -> bool {
        let layout = self.layout(grid.0, grid.1);
        match self.hit(&layout, col, row) {
            Hit::Folder(index) => {
                self.focus = Focus::Sidebar;
                self.select_folder_row(index);
            }
            Hit::Message(index) => {
                self.focus = Focus::List;
                self.select_message(index, &layout);
            }
            Hit::Sidebar => self.focus = Focus::Sidebar,
            Hit::ListChrome => self.focus = Focus::List,
            Hit::Reader => self.focus = Focus::Reader,
            Hit::Nothing => return false,
        }
        true
    }

    fn wheel(&mut self, col: u16, row: u16, rows: i16, grid: (u16, u16)) -> bool {
        if rows == 0 {
            return false;
        }
        let layout = self.layout(grid.0, grid.1);
        let action = if rows < 0 { Action::Up } else { Action::Down };
        let notches = usize::from(rows.unsigned_abs()) * WHEEL_ROWS;
        let target = match self.hit(&layout, col, row) {
            Hit::Folder(_) | Hit::Sidebar => Focus::Sidebar,
            Hit::Message(_) | Hit::ListChrome => Focus::List,
            Hit::Reader => Focus::Reader,
            Hit::Nothing => return false,
        };
        for _ in 0..notches {
            match target {
                Focus::Sidebar => self.move_folder(action, &layout),
                Focus::List => self.move_message(action, &layout),
                Focus::Reader => self.scroll_reader(action, &layout),
            }
        }
        true
    }

    fn hit(&self, layout: &Layout, col: u16, row: u16) -> Hit {
        let rows = visible_rows(&self.mail.folders);
        let spacing = layout.sidebar.as_ref().map(|sidebar| sidebar.spacing);
        let offsets = spacing.map(|spacing| row_offsets(&rows, spacing)).unwrap_or_default();
        let height = folders_height(layout);
        let view = HitView {
            layout,
            folder_offsets: &offsets,
            folders_first: folders_first(
                &rows,
                &offsets,
                self.selection.folder,
                self.scroll.folders,
                height,
            ),
            list_first: self.list_first(layout),
            messages: self.shown_page().map_or(0, |page| page.envelopes.len()),
        };
        hit_at(&view, col, row)
    }

    /// The page of the selected folder, or `None` while another folder's page
    /// is still on screen.
    fn shown_page(&self) -> Option<&ListPage> {
        let folder = self.selection.folder?;
        self.mail.page.as_ref().filter(|page| page.folder == folder)
    }

    fn list_first(&self, layout: &Layout) -> usize {
        let visible = layout.list.as_ref().map_or(0, |list| usize::from(list.rows.height));
        let total = self.shown_page().map_or(0, |page| page.envelopes.len());
        list_first(self.scroll.list, self.selection.message_index, visible, total)
    }

    fn move_folder(&mut self, action: Action, layout: &Layout) {
        let rows = visible_rows(&self.mail.folders);
        if rows.is_empty() {
            return;
        }
        let height = folders_height(layout);
        let current = rows
            .iter()
            .position(|row| Some(row.folder.id) == self.selection.folder)
            .unwrap_or(0);
        let next = step(action, current, rows.len(), usize::from(height));
        let chosen = rows[next].folder.id;
        if self.selection.folder != Some(chosen) {
            self.selection.folder = Some(chosen);
            self.selection.message = None;
            self.selection.message_index = 0;
            self.scroll.list = 0;
            self.scroll.reader = 0;
        }
        let spacing = layout.sidebar.as_ref().map(|sidebar| sidebar.spacing);
        if let Some(spacing) = spacing {
            let offsets = row_offsets(&rows, spacing);
            self.scroll.folders =
                folders_first(&rows, &offsets, self.selection.folder, self.scroll.folders, height);
        }
    }

    fn select_folder_row(&mut self, index: usize) {
        let rows = visible_rows(&self.mail.folders);
        let Some(row) = rows.get(index) else {
            return;
        };
        let chosen = row.folder.id;
        if self.selection.folder != Some(chosen) {
            self.selection.folder = Some(chosen);
            self.selection.message = None;
            self.selection.message_index = 0;
            self.scroll.list = 0;
            self.scroll.reader = 0;
        }
    }

    fn move_message(&mut self, action: Action, layout: &Layout) {
        let visible = layout.list.as_ref().map_or(1, |list| usize::from(list.rows.height));
        let Some(page) = self.shown_page() else {
            return;
        };
        let next = step(action, self.selection.message_index, page.envelopes.len(), visible);
        self.select_message(next, layout);
    }

    fn select_message(&mut self, index: usize, layout: &Layout) {
        let visible = layout.list.as_ref().map_or(1, |list| usize::from(list.rows.height));
        let Some(page) = self.mail.page.as_ref().filter(|p| Some(p.folder) == self.selection.folder)
        else {
            return;
        };
        let Some(envelope) = page.envelopes.get(index) else {
            return;
        };
        let changed = self.selection.message != Some(envelope.id);
        self.selection.message = Some(envelope.id);
        self.selection.message_index = index;
        self.scroll.list = list_first(self.scroll.list, index, visible, page.envelopes.len());
        if changed {
            self.scroll.reader = 0;
        }
    }

    fn scroll_reader(&mut self, action: Action, layout: &Layout) {
        let Some(reader) = &layout.reader else {
            return;
        };
        let Some((id, body)) = &self.mail.body else {
            return;
        };
        if Some(*id) != self.selection.message {
            return;
        }
        let ctx = DrawContext::new(&self.theme, &self.icons, &self.config, self.now);
        let view = BodyView {
            lines: &body.lines,
            first: 0,
            app_name: &self.identity.name,
            rules: HighlightRules::default(),
        };
        let height = usize::from(reader.body.height);
        let last = visual_len(&ctx, reader.body, &view).saturating_sub(height);
        let next = match action {
            Action::Up => self.scroll.reader.saturating_sub(1),
            Action::Down => self.scroll.reader.saturating_add(1),
            Action::PageUp => self.scroll.reader.saturating_sub(height.saturating_sub(1).max(1)),
            Action::PageDown => self.scroll.reader.saturating_add(height.saturating_sub(1).max(1)),
            Action::First => 0,
            Action::Last => last,
            _ => self.scroll.reader,
        };
        self.scroll.reader = next.min(last);
    }
}

/// Into the next pane: from the folders to the list, from the list to the
/// message.
fn deeper(focus: Focus) -> Focus {
    match focus {
        Focus::Sidebar => Focus::List,
        Focus::List | Focus::Reader => Focus::Reader,
    }
}

/// Back out to the previous pane.
fn shallower(focus: Focus) -> Focus {
    match focus {
        Focus::Reader => Focus::List,
        Focus::List | Focus::Sidebar => Focus::Sidebar,
    }
}

/// How many rows the folder section's body has.
fn folders_height(layout: &Layout) -> u16 {
    layout
        .sidebar
        .as_ref()
        .and_then(|sidebar| sidebar.section(SectionKind::Folders))
        .map_or(0, |section| section.body.height)
}

#[cfg(test)]
include!("dispatch.tests.rs");
