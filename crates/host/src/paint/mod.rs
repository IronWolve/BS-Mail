//! Pixels: the canvas, the cell grid, and everything drawn into it.

pub mod block_elements;
pub mod box_drawing;
pub mod canvas;
pub mod caret;
pub mod cell_painter;
pub mod colors;
pub mod context;
pub mod effects;
pub mod glow;
pub mod glyph_draw;
pub mod grid;
pub mod icon_fit;
pub mod mask_cache;
pub mod rounded;
pub mod scaled_text;
pub mod shapes;
pub mod text_mask;

pub use canvas::Canvas;
pub use colors::{Ink, Palette, Rgb};
pub use context::PaintContext;
pub use grid::{Grid, PixelRect};

use surface::Frame;

/// The whole picture, in the order a reader sees it: cell backgrounds, the
/// effects that are backgrounds, every symbol, then the effects that sit on
/// top.
pub fn paint_frame(canvas: &mut Canvas, context: &mut PaintContext<'_>, frame: &Frame) {
    cell_painter::paint_backgrounds(canvas, context, frame);
    effects::paint_under(canvas, context, frame);
    let changes = effects::cell_changes(frame);
    cell_painter::paint_symbols(canvas, context, frame, &changes);
    effects::paint_over(canvas, context, frame);
    if let Some(request) = &frame.cursor {
        caret::draw_caret(canvas, context, request);
    }
}
