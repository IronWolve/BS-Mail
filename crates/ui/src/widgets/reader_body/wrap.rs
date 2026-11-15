//! Wrapping one styled line to a width in cells: at spaces where there is
//! one, inside a word that is longer than the line. A continuation line
//! starts at the left edge again.

use model::{BodyRole, StyledSpan};
use unicode_segmentation::UnicodeSegmentation;

use crate::widgets::cell_text::cells;

/// A piece of one wrapped line with one role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    pub text: String,
    pub role: BodyRole,
}

/// One grapheme with its width and role.
struct Cell {
    symbol: String,
    width: u16,
    role: BodyRole,
}

impl Cell {
    fn is_space(&self) -> bool {
        self.symbol == " "
    }
}

/// Spaces per tab, since a tab has no width of its own in a cell grid.
const TAB: &str = "    ";

/// Whether `mark` is a continuation of `base`: the two make one grapheme, as
/// an accent does with its letter. A control or a bidi override does not.
fn continues(base: &str, mark: &str) -> bool {
    let mut both = String::with_capacity(base.len() + mark.len());
    both.push_str(base);
    both.push_str(mark);
    both.graphemes(true).count() == 1
}

fn flatten(spans: &[StyledSpan]) -> Vec<Cell> {
    let mut out: Vec<Cell> = Vec::new();
    // The last grapheme was kept, so a mark that opens the next span has a
    // character to belong to.
    let mut open = false;
    for span in spans {
        let text = span.text.replace('\t', TAB);
        for (index, grapheme) in text.graphemes(true).enumerate() {
            let width = cells(grapheme);
            if grapheme.chars().any(char::is_control) {
                open = false;
                continue;
            }
            if width == 0 {
                // Highlighting cuts spans at characters, not graphemes, so an
                // accent can open a span of its own: it goes back to the
                // character before it, as it would in one span. A mark with no
                // base, or after something dropped, is left out.
                match out.last_mut() {
                    Some(last) if open && index == 0 && continues(&last.symbol, grapheme) => {
                        last.symbol.push_str(grapheme);
                        last.width = cells(&last.symbol).max(1);
                    }
                    _ => open = false,
                }
                continue;
            }
            open = true;
            out.push(Cell {
                symbol: grapheme.to_string(),
                width,
                role: span.role,
            });
        }
    }
    out
}

/// End (exclusive) of the longest run of cells from `start` that fits.
fn fitting_end(cells: &[Cell], start: usize, width: u16) -> usize {
    let mut used = 0u16;
    let mut end = start;
    while end < cells.len() && used.saturating_add(cells[end].width) <= width {
        used += cells[end].width;
        end += 1;
    }
    end
}

/// Where to break a line that starts at `start` and runs out of room at
/// `end`: the last space after the first word, else a hard cut at `end`
/// (at least one cell, so the loop always moves on). Returns the end of the
/// line and whether the break is at a space.
fn break_at(cells: &[Cell], start: usize, end: usize) -> (usize, bool) {
    let first_word = (start..end).find(|i| !cells[*i].is_space());
    let space = first_word
        .and_then(|word| (word..=end).rev().find(|i| cells.get(*i).is_some_and(Cell::is_space)));
    match space {
        Some(at) => (at, true),
        None => (end.max(start + 1), false),
    }
}

fn pieces(cells: &[Cell]) -> Vec<Piece> {
    let mut out: Vec<Piece> = Vec::new();
    for cell in cells {
        match out.last_mut() {
            Some(last) if last.role == cell.role => last.text.push_str(&cell.symbol),
            _ => out.push(Piece {
                text: cell.symbol.clone(),
                role: cell.role,
            }),
        }
    }
    out
}

/// The line as wrapped pieces, one `Vec` per visual line. An empty line gives
/// one empty visual line; a zero width gives none.
pub fn wrap_line(spans: &[StyledSpan], width: u16) -> Vec<Vec<Piece>> {
    if width == 0 {
        return Vec::new();
    }
    let all = flatten(spans);
    if all.is_empty() {
        return vec![Vec::new()];
    }
    let mut lines = Vec::new();
    let mut start = 0;
    while start < all.len() {
        let end = fitting_end(&all, start, width);
        if end == all.len() {
            lines.push(pieces(&all[start..]));
            break;
        }
        let (stop, at_space) = break_at(&all, start, end);
        let mut shown = stop;
        while at_space && shown > start && all[shown - 1].is_space() {
            shown -= 1;
        }
        lines.push(pieces(&all[start..shown]));
        start = stop;
        while at_space && start < all.len() && all[start].is_space() {
            start += 1;
        }
    }
    lines
}

#[cfg(test)]
include!("wrap.tests.rs");
