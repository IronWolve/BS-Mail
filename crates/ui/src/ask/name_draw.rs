//! The box that asks for a folder's name, on screen: its title, a line that
//! says where the folder goes or which one is renamed, the typed name with its
//! caret, the reason the last name was refused, and the two words on the
//! bottom border that accept and cancel (drawn as keycaps, and found by a
//! click).

use surface::{CursorRequest, CursorShape, Frame, Rect};

use super::name::{CANCEL_WORDS, NameParts, NamePrompt, PROMPT, name_parts, text_skip};
use crate::composer::graphemes::{cells_between, list};
use crate::format::{ELLIPSIS_CHAR, truncate};
use crate::theme::roles;
use crate::widgets::help_overlay::overlaps;
use crate::widgets::{DrawContext, fill_cells, put_text};

pub fn draw_name_prompt(frame: &mut Frame, ctx: &DrawContext<'_>, grid: Rect, prompt: &NamePrompt) {
    let Some(parts) = name_parts(grid, prompt.go) else {
        return;
    };
    let rect = parts.rect;
    frame.effects.retain(|effect| !overlaps(effect.area(), rect));
    fill_cells(frame, rect, ctx.full(roles::MENU_BASE));
    draw_frame(frame, ctx, rect, prompt.title);
    draw_buttons(frame, ctx, &parts, prompt.go);
    let context = truncate(&prompt.context, usize::from(parts.context.width).saturating_sub(2), ELLIPSIS_CHAR);
    put_text(frame, parts.context, parts.context.x + 1, parts.context.y, &context, ctx.fg(roles::FOOTER_TEXT));
    draw_typed(frame, ctx, &parts, prompt);
    if let Some(error) = &prompt.error {
        let shown = truncate(error, usize::from(parts.error.width).saturating_sub(2), ELLIPSIS_CHAR);
        put_text(frame, parts.error, parts.error.x + 1, parts.error.y, &shown, ctx.fg(roles::FOOTER_ERROR));
    }
}

fn draw_typed(frame: &mut Frame, ctx: &DrawContext<'_>, parts: &NameParts, prompt: &NamePrompt) {
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

/// The two buttons on the bottom border: the key as a keycap, the word beside it.
fn draw_buttons(frame: &mut Frame, ctx: &DrawContext<'_>, parts: &NameParts, go: &str) {
    let hint = format!(" {go}   {CANCEL_WORDS} ");
    put_text(frame, parts.rect, parts.go.x.saturating_sub(1), parts.go.y, &hint, ctx.fg(roles::FOOTER_TEXT));
    for (rect, words) in [(parts.go, go), (parts.cancel, CANCEL_WORDS)] {
        let key_cells = u16::try_from(words.find(' ').unwrap_or(0)).unwrap_or(0);
        let cap = &words[..usize::from(key_cells)];
        put_text(frame, parts.rect, rect.x, rect.y, cap, ctx.full(roles::KEYBAR_CAP));
        put_text(frame, parts.rect, rect.x + key_cells, rect.y, &words[usize::from(key_cells)..], ctx.fg(roles::KEYBAR_LABEL));
    }
}

fn draw_frame(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, title: &str) {
    let border = ctx.fg(roles::MENU_BORDER);
    let set = ctx.borders;
    let (left, right) = (rect.x, rect.right() - 1);
    let (top, bottom) = (rect.y, rect.bottom() - 1);
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
}

#[cfg(test)]
include!("name_draw.tests.rs");
