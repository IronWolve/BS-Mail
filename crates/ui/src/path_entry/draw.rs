//! The path box on screen: its title, what it is for, the typed path with its
//! caret, the reason the last path was refused, the list of matching folders
//! and files with the highlighted one, and the two words on the bottom border
//! that accept and cancel.

use surface::{CursorRequest, CursorShape, Frame, Rect};

use super::prompt::{PROMPT, PathParts, PathPrompt, path_parts};
use crate::ask::name::text_skip;
use crate::ask::name_draw::{draw_button_pair, draw_frame};
use crate::composer::graphemes::{cells_between, list};
use crate::format::{ELLIPSIS_CHAR, truncate};
use crate::theme::roles;
use crate::widgets::help_overlay::overlaps;
use crate::widgets::{DrawContext, fill_cells, put_text};

pub fn draw_path_prompt(frame: &mut Frame, ctx: &DrawContext<'_>, grid: Rect, prompt: &PathPrompt) {
    let Some(parts) = path_parts(grid, prompt.go, prompt.visible_rows()) else {
        return;
    };
    let rect = parts.rect;
    frame.effects.retain(|effect| !overlaps(effect.area(), rect));
    fill_cells(frame, rect, ctx.full(roles::MENU_BASE));
    draw_frame(frame, ctx, rect, prompt.title);
    draw_button_pair(frame, ctx, rect, parts.go, parts.cancel, prompt.go);
    let context = truncate(&prompt.context, usize::from(parts.context.width).saturating_sub(2), ELLIPSIS_CHAR);
    put_text(frame, parts.context, parts.context.x + 1, parts.context.y, &context, ctx.fg(roles::FOOTER_TEXT));
    draw_typed(frame, ctx, &parts, prompt);
    if let Some(error) = &prompt.error {
        let shown = truncate(error, usize::from(parts.error.width).saturating_sub(2), ELLIPSIS_CHAR);
        put_text(frame, parts.error, parts.error.x + 1, parts.error.y, &shown, ctx.fg(roles::FOOTER_ERROR));
    }
    draw_matches(frame, ctx, &parts, prompt);
}

fn draw_typed(frame: &mut Frame, ctx: &DrawContext<'_>, parts: &PathParts, prompt: &PathPrompt) {
    let line = &prompt.buffer.lines()[0];
    let col = prompt.buffer.cursor().col;
    put_text(frame, parts.line, parts.line.x + 1, parts.line.y, PROMPT, ctx.fg(roles::LIST_HINT_KEY));
    let skip = text_skip(line, col, parts.room);
    let text: String = list(line).into_iter().skip(skip).collect();
    put_text(frame, parts.line, parts.text_x, parts.line.y, &text, ctx.fg(roles::MENU_BASE));
    let caret = parts.text_x + u16::try_from(cells_between(line, skip, col)).unwrap_or(0);
    if caret < parts.line.right() {
        frame.cursor = Some(CursorRequest { col: caret, row: parts.line.y, shape: CursorShape::Bar });
    }
}

/// The rows of the list, a folder with a separator after its name, the
/// highlighted one in the colours of a selected row.
fn draw_matches(frame: &mut Frame, ctx: &DrawContext<'_>, parts: &PathParts, prompt: &PathPrompt) {
    let shown = prompt.list.iter().skip(prompt.first).take(usize::from(parts.list.height));
    for (offset, found) in (0u16..).zip(shown) {
        let row = Rect::new(parts.list.x, parts.list.y + offset, parts.list.width, 1);
        let chosen = prompt.chosen == Some(prompt.first + usize::from(offset));
        let style = if chosen { ctx.full(roles::SIDEBAR_SELECTED) } else { ctx.full(roles::MENU_BASE) };
        fill_cells(frame, row, style);
        let name = if found.is_dir { format!("{}/", found.name.trim_end_matches('/')) } else { found.name.clone() };
        let text = truncate(&name, usize::from(row.width).saturating_sub(2), ELLIPSIS_CHAR);
        put_text(frame, row, row.x + 1, row.y, &text, style);
    }
}

#[cfg(test)]
include!("draw.tests.rs");
