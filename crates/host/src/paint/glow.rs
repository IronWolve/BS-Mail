//! The soft halo under scaled text.
//!
//! It is the text's own shape, blurred and laid down faintly before the text
//! itself, so the letters keep their hard edges.

use super::canvas::Canvas;
use super::colors::Rgb;
use super::text_mask::TextMask;

/// How strongly the blurred shape shows, out of 255.
pub const STRENGTH: u8 = 150;

/// Lays the blurred shape of `mask` onto the canvas in `color`.
pub fn draw_glow(canvas: &mut Canvas, mask: &TextMask, color: Rgb, radius: u8) {
    if radius == 0 || mask.is_blank() {
        return;
    }
    lay_glow(canvas, &mask.blurred(radius), color);
}

/// Lays a shape that is already blurred onto the canvas in `color`.
pub fn lay_glow(canvas: &mut Canvas, blurred: &TextMask, color: Rgb) {
    let rect = blurred.rect;
    for y in rect.y..rect.bottom() {
        for x in rect.x..rect.right() {
            let coverage = blurred.coverage(x, y);
            if coverage == 0 {
                continue;
            }
            let alpha = (u32::from(coverage) * u32::from(STRENGTH) / 255) as u8;
            canvas.blend(x, y, color, alpha);
        }
    }
}

#[cfg(test)]
include!("glow.tests.rs");
