//! Block and shade characters drawn as rectangles.
//!
//! Gauges and scrollbars are built from these, so they must meet their
//! neighbours exactly and fill whole pixels; a font's own block glyphs often
//! leave a hairline between cells.

use super::canvas::Canvas;
use super::colors::Rgb;
use super::grid::PixelRect;

/// The edge a part-cell block grows from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Block {
    /// A part of the cell filled from one edge, 0 to 1 of its size.
    Part { side: Side, fraction: f32 },
    /// The whole cell at a coverage out of 255.
    Shade(u8),
}

/// Draws a block in one cell.
pub fn draw_block(canvas: &mut Canvas, cell: PixelRect, block: Block, color: Rgb) {
    match block {
        Block::Shade(alpha) => shade(canvas, cell, color, alpha),
        Block::Part { side, fraction } => {
            let part = |size: u32| ((size as f32) * fraction).round().max(0.0) as u32;
            let (x, y, width, height) = match side {
                Side::Left => (cell.x, cell.y, part(cell.width), cell.height),
                Side::Right => {
                    let width = part(cell.width);
                    (cell.right() - width as i32, cell.y, width, cell.height)
                }
                Side::Top => (cell.x, cell.y, cell.width, part(cell.height)),
                Side::Bottom => {
                    let height = part(cell.height);
                    (cell.x, cell.bottom() - height as i32, cell.width, height)
                }
            };
            canvas.fill(x, y, width, height, color);
        }
    }
}

/// A flat wash over the whole cell.
fn shade(canvas: &mut Canvas, cell: PixelRect, color: Rgb, alpha: u8) {
    for y in cell.y..cell.bottom() {
        for x in cell.x..cell.right() {
            canvas.blend(x, y, color, alpha);
        }
    }
}

/// The block a character draws, or `None` when it is not one.
pub fn block_glyph(ch: char) -> Option<Block> {
    let part = |side, eighths: u8| {
        Some(Block::Part {
            side,
            fraction: f32::from(eighths) / 8.0,
        })
    };
    match ch {
        '\u{2588}' => part(Side::Left, 8),
        // Lower blocks, one eighth up to seven eighths.
        '\u{2581}'..='\u{2587}' => part(Side::Bottom, ch as u8 - 0x80),
        '\u{2580}' => part(Side::Top, 4),
        // Left blocks, seven eighths down to one eighth.
        '\u{2589}'..='\u{258f}' => part(Side::Left, 0x90 - ch as u8),
        '\u{2590}' => part(Side::Right, 4),
        '\u{2591}' => Some(Block::Shade(64)),
        '\u{2592}' => Some(Block::Shade(128)),
        '\u{2593}' => Some(Block::Shade(192)),
        '\u{2594}' => part(Side::Top, 1),
        '\u{2595}' => part(Side::Right, 1),
        _ => None,
    }
}

#[cfg(test)]
include!("block_elements.tests.rs");
