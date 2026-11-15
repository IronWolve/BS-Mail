//! The question on screen: a box over the middle of the screen with what will
//! happen and two buttons.

use surface::{Frame, Modifier, Rect};

use super::state::{Choice, DeletePrompt};
use crate::theme::roles;
use crate::widgets::help_overlay::overlaps;
use crate::widgets::{DrawContext, fill_cells, put_text};

pub const DELETE: &str = " Delete for good ";
pub const KEEP: &str = " Keep them ";
const WIDTH: u16 = 60;
const HEIGHT: u16 = 7;
const WARNING: &str = "This removes it from the server for good, with no undo.";
const HINT: &str = " y deletes   n or Esc keeps ";

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

/// The box for a grid, or `None` when the grid is too small for it.
pub fn prompt_rects(grid: Rect) -> Option<PromptRects> {
    let width = WIDTH.min(grid.width.saturating_sub(2));
    if width < cells(DELETE) + cells(KEEP) + 6 || grid.height < HEIGHT {
        return None;
    }
    let frame = Rect::new(grid.x + (grid.width - width) / 2, grid.y + (grid.height - HEIGHT) / 3, width, HEIGHT);
    let row = frame.y + 4;
    let delete = Rect::new(frame.x + 3, row, cells(DELETE), 1);
    let keep = Rect::new(delete.right() + 3, row, cells(KEEP), 1);
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

pub fn draw_delete_prompt(frame: &mut Frame, ctx: &DrawContext<'_>, grid: Rect, prompt: &DeletePrompt) {
    let Some(rects) = prompt_rects(grid) else {
        return;
    };
    let rect = rects.frame;
    frame.effects.retain(|effect| !overlaps(effect.area(), rect));
    fill_cells(frame, rect, ctx.full(roles::MENU_BASE));
    draw_border(frame, ctx, rect);
    let count = prompt.for_good;
    let question = if count == 1 { "Delete this message for good?".to_string() } else { format!("Delete {count} messages for good?") };
    let inner = Rect::new(rect.x + 1, rect.y + 1, rect.width - 2, rect.height - 2);
    put_text(frame, inner, inner.x + 2, rect.y + 2, &question, ctx.fg(roles::FOOTER_ERROR).add_modifier(Modifier::BOLD));
    put_text(frame, inner, inner.x + 2, rect.y + 3, WARNING, ctx.fg(roles::MENU_BASE));
    draw_button(frame, ctx, rects.delete, DELETE, prompt.focus == Choice::Delete);
    draw_button(frame, ctx, rects.keep, KEEP, prompt.focus == Choice::Keep);
}

fn draw_border(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect) {
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
    put_text(frame, rect, left + 2, top, " Delete for good ", ctx.fg(roles::SECTION_TITLE));
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
