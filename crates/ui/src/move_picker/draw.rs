//! The picker on screen: a box over the middle of the screen with the typed
//! text, the folders that match it and the one highlighted.

use model::Folder;
use surface::{CursorRequest, CursorShape, Frame, Rect};

use super::rows::rows_span;
use super::state::MovePicker;
use crate::format::{ELLIPSIS_CHAR, display_width, truncate};
use crate::theme::roles;
use crate::widgets::help_overlay::overlaps;
use crate::widgets::{DrawContext, fill_cells, put_text};

/// Folder rows shown at most; the list scrolls past this.
const MAX_ROWS: u16 = 12;
const WIDTH: u16 = 60;
const HINT: &str = " Enter moves   Esc cancels ";

/// The box for a grid: centered, as wide as is useful and no taller than the
/// folders need.
pub fn picker_rect(grid: Rect, shown: usize) -> Rect {
    let rows = u16::try_from(shown).unwrap_or(MAX_ROWS).clamp(1, MAX_ROWS);
    let width = WIDTH.min(grid.width.saturating_sub(2));
    // Border, prompt, rule, the rows, border.
    let height = (rows + 4).min(grid.height);
    Rect::new(
        grid.x + (grid.width - width) / 2,
        grid.y + (grid.height.saturating_sub(height)) / 3,
        width,
        height,
    )
}

pub fn draw_picker(frame: &mut Frame, ctx: &DrawContext<'_>, grid: Rect, picker: &MovePicker, folders: &[Folder]) {
    let candidates = picker.candidates(folders);
    let rect = picker_rect(grid, candidates.len());
    if rect.width < 12 || rect.height < 5 {
        return;
    }
    frame.effects.retain(|effect| !overlaps(effect.area(), rect));
    fill_cells(frame, rect, ctx.full(roles::MENU_BASE));
    let count = picker.targets.len();
    let title = if count == 1 { " Move 1 message to ".to_string() } else { format!(" Move {count} messages to ") };
    draw_frame(frame, ctx, rect, (&title, HINT));
    let inner = Rect::new(rect.x + 1, rect.y + 1, rect.width - 2, rect.height - 2);
    draw_filter_line(frame, ctx, inner, &picker.filter);
    draw_rows(frame, ctx, (rect, inner), picker, &candidates);
}

/// The border of a picker box with its title on the top edge and its hint on
/// the bottom edge. The tag picker draws the same box.
pub(crate) fn draw_frame(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, (title, hint): (&str, &str)) {
    let border = ctx.fg(roles::MENU_BORDER);
    let set = ctx.borders;
    let (left, right) = (rect.x, rect.right() - 1);
    let (top, bottom) = (rect.y, rect.bottom() - 1);
    let put = |frame: &mut Frame, col: u16, row: u16, ch: char| {
        put_text(frame, rect, col, row, &ch.to_string(), border);
    };
    for col in left + 1..right {
        put(frame, col, top, set.horizontal);
        put(frame, col, bottom, set.horizontal);
    }
    for row in top + 1..bottom {
        put(frame, left, row, set.vertical);
        put(frame, right, row, set.vertical);
    }
    put(frame, left, top, set.top_left);
    put(frame, right, top, set.top_right);
    put(frame, left, bottom, set.bottom_left);
    put(frame, right, bottom, set.bottom_right);
    put_text(frame, rect, left + 2, top, title, ctx.fg(roles::SECTION_TITLE));
    let hint_at = right.saturating_sub(u16::try_from(hint.chars().count()).unwrap_or(0) + 1);
    put_text(frame, rect, hint_at, bottom, hint, ctx.fg(roles::FOOTER_TEXT));
}

/// The typed text after a prompt mark, then a rule; the caret sits after the
/// last typed letter.
pub(crate) fn draw_filter_line(frame: &mut Frame, ctx: &DrawContext<'_>, inner: Rect, filter: &str) {
    let text = format!("> {filter}");
    put_text(frame, inner, inner.x + 1, inner.y, &text, ctx.fg(roles::MENU_BASE));
    let rule = ctx.borders.horizontal.to_string().repeat(usize::from(inner.width));
    put_text(frame, inner, inner.x, inner.y + 1, &rule, ctx.fg(roles::MENU_BORDER));
    let caret = inner.x + 1 + u16::try_from(display_width(&text)).unwrap_or(0);
    if caret < inner.right() {
        frame.cursor = Some(CursorRequest {
            col: caret,
            row: inner.y,
            shape: CursorShape::Bar,
        });
    }
}

fn draw_rows(frame: &mut Frame, ctx: &DrawContext<'_>, (rect, inner): (Rect, Rect), picker: &MovePicker, candidates: &[&Folder]) {
    let (top, room, first) = rows_span(rect, picker.cursor);
    if candidates.is_empty() {
        put_text(frame, inner, inner.x + 2, top, "no folder matches", ctx.fg(roles::FOOTER_TEXT));
        return;
    }
    for (line, folder) in candidates.iter().enumerate().skip(first).take(room) {
        let y = top + u16::try_from(line - first).unwrap_or(0);
        let selected = line == picker.cursor;
        if selected {
            fill_cells(frame, Rect::new(inner.x, y, inner.width, 1), ctx.full(roles::MENU_SELECTED));
        }
        let style = if selected { ctx.full(roles::MENU_SELECTED) } else { ctx.fg(roles::MENU_BASE) };
        let name = if folder.path.is_empty() { &folder.name } else { &folder.path };
        let count = if folder.unread > 0 { format!("[{}]", folder.unread) } else { String::new() };
        let room_for_name = usize::from(inner.width).saturating_sub(display_width(&count) + 4);
        let shown = truncate(name, room_for_name, ELLIPSIS_CHAR);
        put_text(frame, inner, inner.x + 1, y, &shown, style);
        let count_at = inner.right().saturating_sub(u16::try_from(display_width(&count)).unwrap_or(0) + 1);
        put_text(frame, inner, count_at, y, &count, style);
    }
}

#[cfg(test)]
include!("draw.tests.rs");
