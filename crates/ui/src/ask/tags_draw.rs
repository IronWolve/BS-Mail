//! The tag picker on screen: a box over the middle of the screen with the
//! typed text, the tags that match it, each with a mark for how many of the
//! messages hold it, and the one highlighted.

use surface::{Frame, Rect};

use super::tags::{Held, TagPicker, TagRow};
use crate::format::{ELLIPSIS_CHAR, display_width, truncate};
use crate::move_picker::draw::{draw_filter_line, draw_frame, picker_rect};
use crate::move_picker::rows::rows_span;
use crate::theme::roles;
use crate::widgets::help_overlay::overlaps;
use crate::widgets::{DrawContext, fill_cells, put_text};

pub const HINT: &str = " Enter sets or clears   Esc cancels ";

/// The title: how many messages the tag is for.
pub fn title(count: usize) -> String {
    if count == 1 { " Tag 1 message ".to_owned() } else { format!(" Tag {count} messages ") }
}

/// The mark before a row: whether the tag is on all, some or none of the
/// messages, or is a new one.
pub fn mark(row: &TagRow) -> &'static str {
    match (row.is_new, row.held) {
        (true, _) => "+ new tag: ",
        (false, Held::All) => "[x] ",
        (false, Held::Some) => "[-] ",
        (false, Held::NoneOfThem) => "[ ] ",
    }
}

pub fn draw_tag_picker(frame: &mut Frame, ctx: &DrawContext<'_>, grid: Rect, picker: &TagPicker) {
    let rows = picker.rows();
    let rect = picker_rect(grid, rows.len());
    if rect.width < 12 || rect.height < 5 {
        return;
    }
    frame.effects.retain(|effect| !overlaps(effect.area(), rect));
    fill_cells(frame, rect, ctx.full(roles::MENU_BASE));
    draw_frame(frame, ctx, rect, (&title(picker.targets.len()), HINT));
    let inner = Rect::new(rect.x + 1, rect.y + 1, rect.width - 2, rect.height - 2);
    draw_filter_line(frame, ctx, inner, &picker.filter);
    let (top, room, first) = rows_span(rect, picker.cursor);
    if rows.is_empty() {
        let (text, style) = match picker.problem() {
            Some(problem) => (problem, ctx.fg(roles::FOOTER_ERROR)),
            None => ("no tags yet: type a name to make one".to_owned(), ctx.fg(roles::FOOTER_TEXT)),
        };
        let shown = truncate(&text, usize::from(inner.width).saturating_sub(4), ELLIPSIS_CHAR);
        put_text(frame, inner, inner.x + 2, top, &shown, style);
        return;
    }
    for (line, row) in rows.iter().enumerate().skip(first).take(room) {
        let y = top + u16::try_from(line - first).unwrap_or(0);
        let selected = line == picker.cursor;
        if selected {
            fill_cells(frame, Rect::new(inner.x, y, inner.width, 1), ctx.full(roles::MENU_SELECTED));
        }
        let style = if selected { ctx.full(roles::MENU_SELECTED) } else { ctx.fg(roles::MENU_BASE) };
        let text = format!("{}{}", mark(row), row.tag);
        let shown = truncate(&text, usize::from(inner.width).saturating_sub(3), ELLIPSIS_CHAR);
        debug_assert!(display_width(&shown) <= usize::from(inner.width));
        put_text(frame, inner, inner.x + 1, y, &shown, style);
    }
}

#[cfg(test)]
include!("tags_draw.tests.rs");
