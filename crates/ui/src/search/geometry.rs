//! Where the parts of the search box sit: the drawing and the pointer both
//! ask here, so a click lands on what the user sees.

use surface::Rect;

use crate::composer::graphemes::{cells_between, count};

pub const WIDTH: u16 = 72;
pub const HINT: &str = " Enter searches   Esc cancels ";
pub const PROMPT: &str = "/ ";
/// The two buttons' words, as they stand in `HINT`.
pub const GO_WORDS: &str = "Enter searches";
pub const CANCEL_WORDS: &str = "Esc cancels";
/// Narrowest box that is drawn at all.
pub const MIN_WIDTH: u16 = 14;

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

/// Where the parts of a drawn box are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchParts {
    pub rect: Rect,
    /// The one row inside the border that holds the prompt and the query.
    pub line: Rect,
    /// Where the query's first cell is on that row, after the prompt.
    pub text_x: u16,
    /// How many cells the query has before the right border.
    pub room: usize,
    /// The words that run the search, on the bottom border.
    pub go: Rect,
    /// The words that cancel.
    pub cancel: Rect,
}

fn cells(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

/// The parts of the box on `grid`, or `None` when the grid is too small for
/// the box to be drawn.
pub fn search_parts(grid: Rect) -> Option<SearchParts> {
    let rect = search_rect(grid);
    if rect.width < MIN_WIDTH || rect.height < 3 {
        return None;
    }
    let line = Rect::new(rect.x + 1, rect.y + 1, rect.width - 2, 1);
    let prompt = cells(PROMPT);
    let hint_x = (rect.right() - 1).saturating_sub(cells(HINT) + 1);
    let bottom = rect.bottom() - 1;
    let words = |words: &str| {
        let at = HINT.find(words).map_or(0, |byte| cells(&HINT[..byte]));
        Rect::new(hint_x + at, bottom, cells(words), 1)
    };
    Some(SearchParts {
        rect,
        line,
        text_x: line.x + 1 + prompt,
        room: usize::from(line.width.saturating_sub(prompt + 2)),
        go: words(GO_WORDS),
        cancel: words(CANCEL_WORDS),
    })
}

/// How many graphemes of the query scroll off to the left to keep the caret
/// at grapheme `caret` in view in `room` cells.
pub fn text_skip(line: &str, caret: usize, room: usize) -> usize {
    let mut skip = 0;
    while cells_between(line, skip, caret) >= room && skip < count(line) {
        skip += 1;
    }
    skip
}

/// Whether the cell is inside `rect`.
pub fn holds(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.right() && row >= rect.y && row < rect.bottom()
}

#[cfg(test)]
include!("geometry.tests.rs");
