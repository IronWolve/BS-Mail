//! The find box: one row over the bottom of the reading pane's text, with the
//! query, what the find has found (`3 of 17`) and three words that act. The
//! drawing and the pointer both ask `find_parts` where things are, so a click
//! lands on what is seen.

use surface::{CursorRequest, CursorShape, Frame, Rect};

use crate::composer::EditorBuffer;
use crate::composer::graphemes::{cells_between, count, list};
use crate::theme::roles;
use crate::widgets::{DrawContext, fill_cells, put_text};

const LABEL: &str = " Find: ";
const PREVIOUS: &str = " Prev ";
const NEXT: &str = " Next ";
const CLOSE: &str = " Close ";
/// The narrowest bar that shows the three words; narrower ones leave them to the keys.
const WITH_WORDS_FROM: u16 = 48;
/// The narrowest bar that is drawn at all.
const MIN_WIDTH: u16 = 20;

fn width_of(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

/// Where the parts of the find box sit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindParts {
    pub bar: Rect,
    /// Where the query's first cell is.
    pub text_x: u16,
    /// How many cells the query has.
    pub room: usize,
    /// Where the words about the result start.
    pub status_x: u16,
    pub previous: Rect,
    pub next: Rect,
    pub close: Rect,
}

/// The box for the text area `body` of the reading pane, with `status_cells`
/// cells of words about the result; `None` when the pane is too small for it.
pub fn find_parts(body: Rect, status_cells: u16) -> Option<FindParts> {
    if body.height < 3 || body.width < MIN_WIDTH {
        return None;
    }
    // The last column of the body holds its scrollbar.
    let bar = Rect::new(body.x, body.bottom() - 1, body.width - 1, 1);
    let words = bar.width >= WITH_WORDS_FROM;
    let (close_cells, next_cells, previous_cells) = if words {
        (width_of(CLOSE), width_of(NEXT), width_of(PREVIOUS))
    } else {
        (0, 0, 0)
    };
    let close = Rect::new(bar.right() - close_cells, bar.y, close_cells, 1);
    let next = Rect::new(close.x - next_cells, bar.y, next_cells, 1);
    let previous = Rect::new(next.x - previous_cells, bar.y, previous_cells, 1);
    let status_x = previous.x.saturating_sub(status_cells + 1);
    let text_x = bar.x + width_of(LABEL);
    Some(FindParts {
        bar,
        text_x,
        room: usize::from(status_x.saturating_sub(text_x + 1)),
        status_x,
        previous,
        next,
        close,
    })
}

/// How many graphemes of the query scroll off to the left to keep the caret in view.
pub fn text_skip(line: &str, caret: usize, room: usize) -> usize {
    let mut skip = 0;
    while cells_between(line, skip, caret) >= room && skip < count(line) {
        skip += 1;
    }
    skip
}

/// Draws the box. `editing` shows the caret.
pub fn draw_find_bar(frame: &mut Frame, ctx: &DrawContext<'_>, body: Rect, query: &EditorBuffer, status: &str, editing: bool) {
    let Some(parts) = find_parts(body, width_of(status)) else {
        return;
    };
    let bar = parts.bar;
    fill_cells(frame, bar, ctx.full(roles::MENU_BASE));
    put_text(frame, bar, bar.x, bar.y, LABEL, ctx.fg(roles::LIST_HINT_KEY));
    let line = &query.lines()[0];
    let caret = query.cursor().col;
    let skip = text_skip(line, caret, parts.room);
    let shown: String = list(line).into_iter().skip(skip).collect();
    let text_clip = Rect::new(parts.text_x, bar.y, u16::try_from(parts.room).unwrap_or(u16::MAX), 1);
    put_text(frame, text_clip, parts.text_x, bar.y, &shown, ctx.fg(roles::MENU_BASE));
    put_text(frame, bar, parts.status_x, bar.y, status, ctx.fg(roles::FOOTER_TEXT));
    for (rect, words) in [(parts.previous, PREVIOUS), (parts.next, NEXT), (parts.close, CLOSE)] {
        if rect.width > 0 {
            put_text(frame, bar, rect.x, rect.y, words, ctx.full(roles::KEYBAR_CAP));
        }
    }
    if editing {
        let col = parts.text_x + u16::try_from(cells_between(line, skip, caret)).unwrap_or(0);
        if usize::from(col - parts.text_x) <= parts.room {
            frame.cursor = Some(CursorRequest { col, row: bar.y, shape: CursorShape::Bar });
        }
    }
}

#[cfg(test)]
include!("find_bar.tests.rs");
