//! The search box: one line over the top of the list pane for the query.

use surface::{CursorRequest, CursorShape, Frame, Rect};

use super::geometry::{CANCEL_WORDS, GO_WORDS, HINT, PROMPT, SearchParts, search_parts, text_skip};
pub use super::geometry::search_rect;
use crate::composer::EditorBuffer;
use crate::composer::graphemes::{cells_between, list};
use crate::theme::roles;
use crate::widgets::help_overlay::overlaps;
use crate::widgets::{DrawContext, fill_cells, put_text};

const TITLE: &str = " Search ";

pub fn draw_search_bar(frame: &mut Frame, ctx: &DrawContext<'_>, grid: Rect, query: &EditorBuffer) {
    let Some(parts) = search_parts(grid) else {
        return;
    };
    let rect = parts.rect;
    frame.effects.retain(|effect| !overlaps(effect.area(), rect));
    fill_cells(frame, rect, ctx.full(roles::MENU_BASE));
    draw_frame(frame, ctx, rect);
    draw_buttons(frame, ctx, &parts);
    let inner = parts.line;
    put_text(frame, inner, inner.x + 1, inner.y, PROMPT, ctx.fg(roles::LIST_HINT_KEY));
    let line = &query.lines()[0];
    let col = query.cursor().col;
    let skip = text_skip(line, col, parts.room);
    let text: String = list(line).into_iter().skip(skip).collect();
    put_text(frame, inner, parts.text_x, inner.y, &text, ctx.fg(roles::MENU_BASE));
    let caret = parts.text_x + u16::try_from(cells_between(line, skip, col)).unwrap_or(0);
    if caret < inner.right() {
        frame.cursor = Some(CursorRequest {
            col: caret,
            row: inner.y,
            shape: CursorShape::Bar,
        });
    }
}

/// The two buttons on the bottom border: the key as a keycap, the word beside it.
fn draw_buttons(frame: &mut Frame, ctx: &DrawContext<'_>, parts: &SearchParts) {
    for (rect, words) in [(parts.go, GO_WORDS), (parts.cancel, CANCEL_WORDS)] {
        let key_cells = u16::try_from(words.find(' ').unwrap_or(0)).unwrap_or(0);
        let cap = &words[..usize::from(key_cells)];
        put_text(frame, parts.rect, rect.x, rect.y, cap, ctx.full(roles::KEYBAR_CAP));
        put_text(frame, parts.rect, rect.x + key_cells, rect.y, &words[usize::from(key_cells)..], ctx.fg(roles::KEYBAR_LABEL));
    }
}

fn draw_frame(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect) {
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
    put(frame, left, top + 1, set.vertical);
    put(frame, right, top + 1, set.vertical);
    put(frame, left, top, set.top_left);
    put(frame, right, top, set.top_right);
    put(frame, left, bottom, set.bottom_left);
    put(frame, right, bottom, set.bottom_right);
    put_text(frame, rect, left + 2, top, TITLE, ctx.fg(roles::SECTION_TITLE));
    let hint_at = right.saturating_sub(u16::try_from(HINT.chars().count()).unwrap_or(0) + 1);
    put_text(frame, rect, hint_at, bottom, HINT, ctx.fg(roles::FOOTER_TEXT));
}

#[cfg(test)]
include!("draw.tests.rs");
