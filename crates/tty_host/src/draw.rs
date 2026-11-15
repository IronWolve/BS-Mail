//! Frames into a live terminal, writing only what changed.
//!
//! The first frame, and the first after a resize, repaint everything. After
//! that each row is compared with the last frame and only the runs of changed
//! cells are written, each after a move to its place, so a keypress costs a
//! few dozen bytes and not a screenful.

use std::io::{self, Write};

use surface::{ColorDepth, CursorShape, Frame, Style};
use unicode_width::UnicodeWidthStr;

use crate::ansi::{RESET, sgr_at};

#[derive(Debug)]
pub struct Painter {
    previous: Option<Frame>,
    depth: ColorDepth,
}

impl Painter {
    pub fn new(depth: ColorDepth) -> Self {
        Self { previous: None, depth }
    }

    /// Makes the next frame repaint everything, for when the terminal's
    /// contents may no longer be what was last drawn.
    pub fn forget(&mut self) {
        self.previous = None;
    }

    /// Writes `frame`, or what differs from the one before it.
    pub fn paint(&mut self, frame: &Frame, out: &mut impl Write) -> io::Result<()> {
        let same_size = self.previous.as_ref().is_some_and(|last| last.area() == frame.area());
        if !same_size {
            // Home and clear, so nothing from a larger screen is left over.
            out.write_all(b"\x1b[0m\x1b[2J")?;
        }
        let area = frame.area();
        let mut current: Option<Style> = None;
        for row in area.top()..area.bottom() {
            let changed = self.changed_columns(frame, row, same_size);
            let mut col = area.left();
            while col < area.right() {
                if !changed[usize::from(col - area.left())] {
                    col += 1;
                    continue;
                }
                write!(out, "\x1b[{};{}H", row - area.top() + 1, col - area.left() + 1)?;
                col = self.write_run(out, frame, (col, row), &changed, &mut current)?;
            }
        }
        out.write_all(RESET.as_bytes())?;
        write_cursor(out, frame)?;
        out.flush()?;
        self.previous = Some(frame.clone());
        Ok(())
    }

    /// Which columns of `row` must be written. A wide symbol and the cell it
    /// covers are written together, so if either changed both are. The cell a
    /// wide symbol covers holds no symbol of its own whatever it reads back
    /// as (the UI resets it, which reads as a blank), so it is found by the
    /// symbol before it.
    fn changed_columns(&self, frame: &Frame, row: u16, same_size: bool) -> Vec<bool> {
        let area = frame.area();
        let width = usize::from(area.width);
        let Some(last) = self.previous.as_ref().filter(|_| same_size) else {
            return vec![true; width];
        };
        let mut changed: Vec<bool> = (area.left()..area.right())
            .map(|col| frame.cells.cell((col, row)) != last.cells.cell((col, row)))
            .collect();
        for index in 0..width {
            if !changed[index] {
                continue;
            }
            let col = area.left() + u16::try_from(index).unwrap_or(0);
            let wide_here = frame.cells.cell((col, row)).is_some_and(|c| UnicodeWidthStr::width(c.symbol()) >= 2);
            let covered = index > 0
                && frame
                    .cells
                    .cell((col - 1, row))
                    .is_some_and(|before| UnicodeWidthStr::width(before.symbol()) >= 2);
            if wide_here && index + 1 < width {
                changed[index + 1] = true;
            }
            if covered {
                changed[index - 1] = true;
            }
        }
        changed
    }

    /// Writes consecutive changed cells from `start`; returns the column
    /// after the run.
    fn write_run(
        &self,
        out: &mut impl Write,
        frame: &Frame,
        start: (u16, u16),
        changed: &[bool],
        current: &mut Option<Style>,
    ) -> io::Result<u16> {
        let area = frame.area();
        let (mut col, row) = start;
        while col < area.right() && changed[usize::from(col - area.left())] {
            let Some(cell) = frame.cells.cell((col, row)) else {
                break;
            };
            let symbol = cell.symbol();
            // The columns a symbol takes: a wide one gives its columns to the
            // terminal, so the cells it covers are stepped over, not written.
            let columns = u16::try_from(UnicodeWidthStr::width(symbol)).unwrap_or(1).max(1);
            if !symbol.is_empty() {
                let style = Style::new().fg(cell.fg).bg(cell.bg).add_modifier(cell.modifier);
                if *current != Some(style) {
                    out.write_all(sgr_at(&style, self.depth).as_bytes())?;
                    *current = Some(style);
                }
                out.write_all(symbol.as_bytes())?;
            }
            col = col.saturating_add(columns);
        }
        Ok(col)
    }
}

/// Moves the cursor to the caret the screen asks for and shows it, or hides
/// it when there is none.
fn write_cursor(out: &mut impl Write, frame: &Frame) -> io::Result<()> {
    let Some(caret) = frame.cursor else {
        return out.write_all(b"\x1b[?25l");
    };
    let shape = match caret.shape {
        CursorShape::Block => 2,
        CursorShape::Underline => 4,
        CursorShape::Bar => 6,
    };
    write!(out, "\x1b[{};{}H\x1b[{shape} q\x1b[?25h", caret.row + 1, caret.col + 1)
}

#[cfg(test)]
include!("draw.tests.rs");
