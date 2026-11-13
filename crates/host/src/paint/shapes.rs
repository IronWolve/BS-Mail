//! Anti-aliased shapes: a small transparent bitmap is drawn into and then
//! blended onto the canvas. Lines, arcs and rounded rectangles go through
//! here, so curves have a true arc at every cell size.

use tiny_skia::{Paint, Pixmap};

use super::canvas::Canvas;
use super::colors::Rgb;
use super::grid::PixelRect;

/// A paint that covers with one solid color, anti-aliased.
pub fn solid(color: Rgb) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(color.r, color.g, color.b, 255);
    paint.anti_alias = true;
    paint
}

/// Draws into a bitmap the size of `rect` and blends it onto the canvas.
/// The closure's coordinates start at the rectangle's top left corner.
pub fn with_bitmap(canvas: &mut Canvas, rect: PixelRect, draw: impl FnOnce(&mut Pixmap)) {
    let Some(mut bitmap) = Pixmap::new(rect.width.max(1), rect.height.max(1)) else {
        return;
    };
    draw(&mut bitmap);
    blend(canvas, rect, &bitmap);
}

/// Every covered pixel of `bitmap`, over the canvas at `rect`.
fn blend(canvas: &mut Canvas, rect: PixelRect, bitmap: &Pixmap) {
    let width = bitmap.width();
    for (index, pixel) in bitmap.pixels().iter().enumerate() {
        let alpha = pixel.alpha();
        if alpha == 0 {
            continue;
        }
        let color = pixel.demultiply();
        let x = rect.x + (index as u32 % width) as i32;
        let y = rect.y + (index as u32 / width) as i32;
        canvas.blend(x, y, Rgb::new(color.red(), color.green(), color.blue()), alpha);
    }
}
