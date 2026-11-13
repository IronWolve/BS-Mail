//! A whole frame as the lines a terminal prints: one line per grid row, with
//! a style sequence wherever the style changes and a reset at each line end.
//!
//! Effects are not painted here. Every widget writes its text into the cells
//! as well, so a terminal shows the same screen without them: the banner name
//! at normal size instead of double, flat fills instead of rounded ones.

use surface::{Frame, Style};
use unicode_width::UnicodeWidthStr;

use super::sgr::{RESET, sgr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaintOptions {
    /// With colors off, only the symbols are written.
    pub color: bool,
}

impl Default for PaintOptions {
    fn default() -> Self {
        Self { color: true }
    }
}

/// The frame as text, each row followed by a newline.
pub fn paint_frame(frame: &Frame, options: &PaintOptions) -> String {
    let area = frame.area();
    let mut out = String::new();
    for row in area.top()..area.bottom() {
        paint_row(&mut out, frame, row, options);
        out.push('\n');
    }
    out
}

fn paint_row(out: &mut String, frame: &Frame, row: u16, options: &PaintOptions) {
    let area = frame.area();
    let mut col = area.left();
    let mut current: Option<Style> = None;
    while col < area.right() {
        let Some(cell) = frame.cells.cell((col, row)) else {
            break;
        };
        let symbol = cell.symbol();
        // The cells a wide symbol covers carry no symbol of their own; the
        // terminal gives it those columns, so nothing is written for them.
        if !symbol.is_empty() {
            let style = Style::new().fg(cell.fg).bg(cell.bg).add_modifier(cell.modifier);
            if options.color && current != Some(style) {
                out.push_str(&sgr(&style));
                current = Some(style);
            }
            out.push_str(symbol);
        }
        col += if UnicodeWidthStr::width(symbol) >= 2 { 2 } else { 1 };
    }
    if options.color && current.is_some() {
        out.push_str(RESET);
    }
}

#[cfg(test)]
include!("paint.tests.rs");
