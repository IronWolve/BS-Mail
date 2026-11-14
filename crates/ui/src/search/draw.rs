//! The search box: one line over the top of the list pane for the query.

use surface::{CursorRequest, CursorShape, Frame, Rect};

use crate::composer::EditorBuffer;
use crate::composer::graphemes::{cells_between, count, list};
use crate::theme::roles;
use crate::widgets::help_overlay::overlaps;
use crate::widgets::{DrawContext, fill_cells, put_text};

const WIDTH: u16 = 72;
const TITLE: &str = " Search ";
const HINT: &str = " Enter searches   Esc cancels ";
const PROMPT: &str = "/ ";

/// The box for a grid: centered and three rows high.
pub fn search_rect(grid: Rect) -> Rect {
    let width = WIDTH.min(grid.width.saturating_sub(2));
    let height = 3.min(grid.height);
    Rect::new(
        grid.x + (grid.width - width) / 2,
        grid.y + grid.height.saturating_sub(height) / 4,
        width,
        height,
    )
}

pub fn draw_search_bar(frame: &mut Frame, ctx: &DrawContext<'_>, grid: Rect, query: &EditorBuffer) {
    let rect = search_rect(grid);
    if rect.width < 14 || rect.height < 3 {
        return;
    }
    frame.effects.retain(|effect| !overlaps(effect.area(), rect));
    fill_cells(frame, rect, ctx.full(roles::MENU_BASE));
    draw_frame(frame, ctx, rect);
    let inner = Rect::new(rect.x + 1, rect.y + 1, rect.width - 2, 1);
    let prompt_cells = u16::try_from(PROMPT.len()).unwrap_or(2);
    put_text(frame, inner, inner.x + 1, inner.y, PROMPT, ctx.fg(roles::LIST_HINT_KEY));
    let room = usize::from(inner.width.saturating_sub(prompt_cells + 2));
    let line = &query.lines()[0];
    let col = query.cursor().col;
    let mut skip = 0;
    while cells_between(line, skip, col) >= room && skip < count(line) {
        skip += 1;
    }
    let text: String = list(line).into_iter().skip(skip).collect();
    put_text(frame, inner, inner.x + 1 + prompt_cells, inner.y, &text, ctx.fg(roles::MENU_BASE));
    let caret = inner.x + 1 + prompt_cells + u16::try_from(cells_between(line, skip, col)).unwrap_or(0);
    if caret < inner.right() {
        frame.cursor = Some(CursorRequest {
            col: caret,
            row: inner.y,
            shape: CursorShape::Bar,
        });
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
