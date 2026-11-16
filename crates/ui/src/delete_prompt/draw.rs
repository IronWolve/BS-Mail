//! The question on screen: a box over the middle of the screen with what will
//! happen (a move to the Trash, or a delete for good) and two buttons.

use surface::{Frame, Modifier, Rect};

use super::state::{Choice, DeletePrompt};
use crate::theme::roles;
use crate::widgets::help_overlay::overlaps;
use crate::widgets::{DrawContext, fill_cells, put_text};

/// The first button when some of the mail is removed for good.
pub const DELETE: &str = " Delete for good ";
/// The first button when all of it only moves to the Trash.
pub const MOVE_TO_TRASH: &str = " Move to Trash ";
pub const KEEP: &str = " Keep them ";
const TITLE: &str = " Delete mail ";
const WIDTH: u16 = 70;
const HEIGHT: u16 = 7;
const WARNING: &str = "This cannot be undone.";
/// Said instead when some of the messages only move to the Trash.
const WARNING_MIXED: &str = "Removed for good: no undo. The rest can be got back.";
const HINT: &str = " y yes   n or Esc keeps ";

/// Where the box and its buttons sit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptRects {
    pub frame: Rect,
    pub delete: Rect,
    pub keep: Rect,
}

fn cells(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(0)
}

/// The text of the first button: what a yes does.
pub fn confirm_label(for_good: usize) -> &'static str {
    if for_good == 0 { MOVE_TO_TRASH } else { DELETE }
}

/// The box for a grid whose first button reads `confirm`, or `None` when the
/// grid is too small for it.
pub fn prompt_rects(grid: Rect, confirm: &str) -> Option<PromptRects> {
    prompt_rects_for(grid, confirm, KEEP)
}

/// The box for a question whose buttons read `confirm` and `keep`.
pub fn prompt_rects_for(grid: Rect, confirm: &str, keep: &str) -> Option<PromptRects> {
    let width = WIDTH.min(grid.width.saturating_sub(2));
    if width < cells(confirm) + cells(keep) + 6 || grid.height < HEIGHT {
        return None;
    }
    let frame = Rect::new(grid.x + (grid.width - width) / 2, grid.y + (grid.height - HEIGHT) / 3, width, HEIGHT);
    let row = frame.y + 4;
    let delete = Rect::new(frame.x + 3, row, cells(confirm), 1);
    let keep = Rect::new(delete.right() + 3, row, cells(keep), 1);
    Some(PromptRects { frame, delete, keep })
}

/// The button under a cell, if any.
pub fn button_at(rects: &PromptRects, col: u16, row: u16) -> Option<Choice> {
    let inside = |rect: Rect| row == rect.y && col >= rect.x && col < rect.right();
    if inside(rects.delete) {
        Some(Choice::Delete)
    } else if inside(rects.keep) {
        Some(Choice::Keep)
    } else {
        None
    }
}

/// The words of a question box and which button has focus.
pub struct BoxText<'a> {
    pub title: &'a str,
    pub question: &'a str,
    pub warning: &'a str,
    /// The button that does it, and the one that keeps things as they are.
    pub confirm: &'a str,
    pub keep: &'a str,
    pub focus: Choice,
}

pub fn draw_delete_prompt(frame: &mut Frame, ctx: &DrawContext<'_>, grid: Rect, prompt: &DeletePrompt) {
    let to_trash = prompt.targets.len().saturating_sub(prompt.for_good);
    let text = BoxText {
        title: TITLE,
        question: &question(prompt.for_good, to_trash),
        warning: warning(prompt.for_good, to_trash),
        confirm: confirm_label(prompt.for_good),
        keep: KEEP,
        focus: prompt.focus,
    };
    draw_box(frame, ctx, grid, &text);
}

/// The question box over the middle of the screen: its title, the question in
/// the warning colour, the line under it and the two buttons.
pub fn draw_box(frame: &mut Frame, ctx: &DrawContext<'_>, grid: Rect, text: &BoxText<'_>) {
    let Some(rects) = prompt_rects_for(grid, text.confirm, text.keep) else {
        return;
    };
    let rect = rects.frame;
    frame.effects.retain(|effect| !overlaps(effect.area(), rect));
    fill_cells(frame, rect, ctx.full(roles::MENU_BASE));
    draw_border(frame, ctx, rect, text.title);
    let inner = Rect::new(rect.x + 1, rect.y + 1, rect.width - 2, rect.height - 2);
    put_text(frame, inner, inner.x + 2, rect.y + 2, text.question, ctx.fg(roles::FOOTER_ERROR).add_modifier(Modifier::BOLD));
    put_text(frame, inner, inner.x + 2, rect.y + 3, text.warning, ctx.fg(roles::MENU_BASE));
    draw_button(frame, ctx, rects.delete, text.confirm, text.focus == Choice::Delete);
    draw_button(frame, ctx, rects.keep, text.keep, text.focus == Choice::Keep);
}

/// What is asked: how many messages move to the Trash and how many are
/// removed for good. Saying only one of the two would let a yes do what the
/// question never named.
pub(super) fn question(for_good: usize, to_trash: usize) -> String {
    let messages = |count: usize| if count == 1 { "1 message".to_string() } else { format!("{count} messages") };
    match (for_good, to_trash) {
        (0, moved) => format!("Move {} to the Trash?", messages(moved)),
        (1, 0) => "Delete this message for good?".to_string(),
        (count, 0) => format!("Delete {count} messages for good?"),
        (count, 1) => format!("Delete {} for good and move 1 other to the Trash?", messages(count)),
        (count, moved) => format!("Delete {} for good and move {moved} others to the Trash?", messages(count)),
    }
}

/// The second line: that a delete for good has no way back, or that a move to
/// the Trash has.
pub(super) fn warning(for_good: usize, to_trash: usize) -> &'static str {
    match (for_good, to_trash) {
        (0, 1) => "You can get it back from the Trash.",
        (0, _) => "You can get them back from the Trash.",
        (_, 0) => WARNING,
        _ => WARNING_MIXED,
    }
}

fn draw_border(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, title: &str) {
    let border = ctx.fg(roles::MENU_BORDER);
    let set = ctx.borders;
    let (left, right, top, bottom) = (rect.x, rect.right() - 1, rect.y, rect.bottom() - 1);
    let put = |frame: &mut Frame, col: u16, row: u16, ch: char| put_text(frame, rect, col, row, &ch.to_string(), border);
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
    let hint_at = right.saturating_sub(cells(HINT) + 1);
    put_text(frame, rect, hint_at, bottom, HINT, ctx.fg(roles::FOOTER_TEXT));
}

fn draw_button(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, text: &str, focused: bool) {
    let style = if focused {
        ctx.full(roles::SIDEBAR_SELECTED).add_modifier(Modifier::BOLD)
    } else {
        ctx.full(roles::KEYBAR_CAP)
    };
    fill_cells(frame, rect, style);
    put_text(frame, rect, rect.x, rect.y, text, style);
}

#[cfg(test)]
include!("draw.tests.rs");
