//! The text caret: where the next letter typed will go.
//!
//! Drawn last, over everything, in the screen's text color. A bar is two
//! pixels wide at the left edge of its cell so it sits between letters.

use surface::{CursorRequest, CursorShape};

use super::canvas::Canvas;
use super::colors::Ink;
use super::context::PaintContext;
use surface::Color;

/// Pixels the bar leaves free at the top and bottom of its cell.
const INSET: i32 = 2;
/// How much of the cell a block caret covers, out of 255, so the letter under
/// it stays readable.
const BLOCK_ALPHA: u8 = 120;

pub fn draw_caret(canvas: &mut Canvas, context: &PaintContext<'_>, request: &CursorRequest) {
    let cell = context.grid.cell(request.col, request.row);
    let color = context.palette.resolve(Color::Reset, Ink::Foreground);
    let stroke = context.grid.metrics.stroke.max(2);
    match request.shape {
        CursorShape::Bar => {
            let height = (cell.height as i32 - INSET * 2).max(1) as u32;
            canvas.fill(cell.x, cell.y + INSET, stroke, height, color);
        }
        CursorShape::Underline => {
            canvas.fill(cell.x, cell.bottom() - stroke as i32, cell.width, stroke, color);
        }
        CursorShape::Block => {
            for y in cell.y..cell.bottom() {
                for x in cell.x..cell.right() {
                    canvas.blend(x, y, color, BLOCK_ALPHA);
                }
            }
        }
    }
}

#[cfg(test)]
include!("caret.tests.rs");
