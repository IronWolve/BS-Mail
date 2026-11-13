//! Writing text into the frame: clipped to a region, one grapheme per cell
//! run, wide characters given their full width, styles set exactly.

use surface::{Frame, Modifier, Rect, Style};
use unicode_segmentation::UnicodeSegmentation;

use crate::format::{ELLIPSIS_CHAR, display_width, take_width, truncate};

/// A piece of text with one style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub text: String,
    pub style: Style,
}

impl Run {
    pub fn new(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    /// Width in cells.
    pub fn width(&self) -> u16 {
        cells(&self.text)
    }
}

/// Width of `text` in cells, capped to a column count.
pub fn cells(text: &str) -> u16 {
    u16::try_from(display_width(text)).unwrap_or(u16::MAX)
}

/// Total width of runs placed one after another.
pub fn runs_width(runs: &[Run]) -> u16 {
    runs.iter().fold(0u16, |sum, run| sum.saturating_add(run.width()))
}

/// The style with every modifier it does not name switched off, so text
/// written over a bold cell does not stay bold.
fn exact(style: Style) -> Style {
    Style {
        sub_modifier: Modifier::all() - style.add_modifier,
        ..style
    }
}

/// Writes `text` from `(col, row)`, clipped to `clip`; a character that would
/// cross the right edge is left out. Returns the column after the last cell
/// written. Control characters and zero-width marks are skipped.
pub fn put_text(frame: &mut Frame, clip: Rect, col: u16, row: u16, text: &str, style: Style) -> u16 {
    if row < clip.top() || row >= clip.bottom() || col >= clip.right() {
        return col;
    }
    let style = exact(style);
    let mut x = col;
    for grapheme in text.graphemes(true) {
        let width = cells(grapheme);
        if width == 0 || grapheme.chars().any(char::is_control) {
            continue;
        }
        if x.saturating_add(width) > clip.right() {
            break;
        }
        if x >= clip.left() {
            write_cell(frame, x, row, grapheme, width, style);
        }
        x += width;
    }
    x
}

fn write_cell(frame: &mut Frame, x: u16, row: u16, symbol: &str, width: u16, style: Style) {
    if let Some(cell) = frame.cells.cell_mut((x, row)) {
        cell.set_symbol(symbol).set_style(style);
    }
    for extra in 1..width {
        if let Some(cell) = frame.cells.cell_mut((x + extra, row)) {
            // The cell keeps the background it had when the style brings none.
            let bg = cell.bg;
            cell.reset();
            cell.set_style(style);
            if style.bg.is_none() {
                cell.bg = bg;
            }
        }
    }
}

/// Writes `text` cut to `max_width` cells with an ellipsis; returns the next
/// column.
pub fn put_cut(
    frame: &mut Frame,
    clip: Rect,
    col: u16,
    row: u16,
    text: &str,
    max_width: u16,
    style: Style,
) -> u16 {
    let cut = truncate(text, usize::from(max_width), ELLIPSIS_CHAR);
    put_text(frame, clip, col, row, &cut, style)
}

/// Writes runs one after another; returns the next column.
pub fn put_runs(frame: &mut Frame, clip: Rect, col: u16, row: u16, runs: &[Run]) -> u16 {
    runs.iter()
        .fold(col, |at, run| put_text(frame, clip, at, row, &run.text, run.style))
}

/// Where text sits inside a slot that is wider than the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

/// Writes `text` inside the slot of `width` cells that starts at `(x, row)`:
/// cut with an ellipsis when too wide, else placed by `align` (a centered
/// text with an odd number of spare cells leans left).
pub fn put_aligned(
    frame: &mut Frame,
    clip: Rect,
    (x, row, width): (u16, u16, u16),
    text: &str,
    align: Align,
    style: Style,
) {
    let cut = truncate(text, usize::from(width), ELLIPSIS_CHAR);
    let spare = width.saturating_sub(cells(&cut));
    let shift = match align {
        Align::Left => 0,
        Align::Center => spare / 2,
        Align::Right => spare,
    };
    put_text(frame, clip, x.saturating_add(shift), row, &cut, style);
}

/// The runs cut to `max_width` cells: text that fits is returned whole,
/// otherwise the last kept cells are followed by an ellipsis in the style of
/// the last run that kept anything.
pub fn cut_runs(runs: &[Run], max_width: u16) -> Vec<Run> {
    if runs_width(runs) <= max_width {
        return runs.to_vec();
    }
    let budget = max_width.saturating_sub(cells(ELLIPSIS_CHAR));
    let mut kept: Vec<Run> = Vec::new();
    let mut used = 0u16;
    for run in runs {
        let room = usize::from(budget - used);
        let piece = take_width(&run.text, room);
        let width = cells(&piece);
        if width > 0 {
            kept.push(Run::new(piece, run.style));
            used += width;
        }
        if width < run.width() {
            break;
        }
    }
    if max_width > 0 {
        let style = kept.last().or(runs.first()).map(|run| run.style).unwrap_or_default();
        kept.push(Run::new(ELLIPSIS_CHAR, style));
    }
    kept
}

/// Writes runs cut to `max_width` cells; returns the next column.
pub fn put_runs_cut(
    frame: &mut Frame,
    clip: Rect,
    col: u16,
    row: u16,
    runs: &[Run],
    max_width: u16,
) -> u16 {
    put_runs(frame, clip, col, row, &cut_runs(runs, max_width))
}

/// Blanks every cell of `rect` and gives it `style`.
pub fn fill_cells(frame: &mut Frame, rect: Rect, style: Style) {
    let style = exact(style);
    let area = frame.area().intersection(rect);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = frame.cells.cell_mut((x, y)) {
                cell.set_symbol(" ").set_style(style);
            }
        }
    }
}

#[cfg(test)]
include!("cell_text.tests.rs");
