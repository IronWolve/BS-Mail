//! One rasterized glyph onto the canvas.
//!
//! A mask is painted in the cell's text color; a color bitmap keeps its own
//! colors. Both are clipped to a rectangle, so a glyph wider than its slot is
//! cut instead of spilling into the next region.

use crate::font::{BitmapContent, GlyphBitmap};

use super::canvas::Canvas;
use super::colors::Rgb;
use super::grid::PixelRect;

/// Draws `bitmap` with its pen on the baseline at `pen`, clipped to `clip`.
pub fn draw_glyph(
    canvas: &mut Canvas,
    bitmap: &GlyphBitmap,
    pen: (i32, i32),
    color: Rgb,
    clip: PixelRect,
) {
    let left = pen.0 + bitmap.left;
    let top = pen.1 - bitmap.top;
    for y in 0..bitmap.height {
        let canvas_y = top + y as i32;
        for x in 0..bitmap.width {
            let canvas_x = left + x as i32;
            if !clip.contains(canvas_x, canvas_y) {
                continue;
            }
            match bitmap.content {
                BitmapContent::Mask => {
                    canvas.blend(canvas_x, canvas_y, color, bitmap.coverage(x, y));
                }
                BitmapContent::Color => {
                    let [r, g, b, a] = bitmap.color(x, y);
                    canvas.blend(canvas_x, canvas_y, Rgb::new(r, g, b), a);
                }
            }
        }
    }
}

#[cfg(test)]
include!("glyph_draw.tests.rs");
