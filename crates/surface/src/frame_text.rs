//! A plain-text view of a frame, for tests and the `--dump-frame` flag: one
//! line per row with wide characters counted, plus a lookup of one cell's
//! symbol and resolved style by column and row.

use unicode_width::UnicodeWidthStr;

use crate::frame::{Color, Frame, Modifier};

/// What one cell shows: its symbol and the style it resolved to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellView {
    pub symbol: String,
    pub fg: Color,
    pub bg: Color,
    pub modifier: Modifier,
}

impl CellView {
    pub fn is_bold(&self) -> bool {
        self.modifier.contains(Modifier::BOLD)
    }

    pub fn is_italic(&self) -> bool {
        self.modifier.contains(Modifier::ITALIC)
    }

    pub fn is_underlined(&self) -> bool {
        self.modifier.contains(Modifier::UNDERLINED)
    }
}

/// The cell at `(col, row)`, or `None` outside the frame.
pub fn cell_at(frame: &Frame, col: u16, row: u16) -> Option<CellView> {
    let cell = frame.cells.cell((col, row))?;
    Some(CellView {
        symbol: cell.symbol().to_string(),
        fg: cell.fg,
        bg: cell.bg,
        modifier: cell.modifier,
    })
}

/// The text of columns `from..to` (the end is excluded) on `row`, as the
/// screen shows it. A wide character is written once, so the text can hold
/// fewer characters than the range has cells; blanks are kept. Cells outside
/// the frame add nothing.
pub fn text_between(frame: &Frame, row: u16, from: u16, to: u16) -> String {
    let area = frame.area();
    let mut out = String::new();
    let mut col = from.max(area.left());
    let end = to.min(area.right());
    while col < end && row >= area.top() && row < area.bottom() {
        let Some(cell) = frame.cells.cell((col, row)) else {
            break;
        };
        let symbol = cell.symbol();
        out.push_str(symbol);
        col += if UnicodeWidthStr::width(symbol) >= 2 { 2 } else { 1 };
    }
    out
}

/// The text of one whole row, trailing spaces kept.
pub fn row_text(frame: &Frame, row: u16) -> String {
    let area = frame.area();
    text_between(frame, row, area.left(), area.right())
}

/// Every row of the frame, each followed by a newline.
pub fn dump_text(frame: &Frame) -> String {
    let area = frame.area();
    let mut out = String::new();
    for row in area.top()..area.bottom() {
        out.push_str(&row_text(frame, row));
        out.push('\n');
    }
    out
}

#[cfg(test)]
include!("frame_text.tests.rs");
