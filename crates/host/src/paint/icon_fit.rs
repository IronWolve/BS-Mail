//! Icons scaled to their slot.
//!
//! An icon is drawn at whatever size fills the slot it was given, not at the
//! text size, and sits in the middle of the slot rather than on the baseline,
//! because it is a picture and not a letter.

use crate::font::{Glyph, GlyphBitmap};

use super::canvas::Canvas;
use super::colors::Rgb;
use super::context::PaintContext;
use super::glyph_draw::draw_glyph;
use super::grid::PixelRect;

/// How much of the slot an icon fills; the rest is breathing room.
const FILL: f32 = 0.9;
/// Sizes are rounded to halves of a pixel, so the cache holds few entries.
const STEP: f32 = 2.0;

/// Draws `glyph` centered in `slot`, scaled to fill it.
pub fn draw_icon(
    canvas: &mut Canvas,
    context: &mut PaintContext<'_>,
    slot: PixelRect,
    glyph: Glyph,
    color: Rgb,
) {
    let base = context.pixel_size();
    let Some(size) = fitted_size(context, slot, glyph, base) else {
        return;
    };
    let Some(bitmap) = context.cache.glyph(context.fonts, glyph.face, glyph.id, size) else {
        return;
    };
    let pen = centered_pen(slot, bitmap);
    draw_glyph(canvas, bitmap, pen, color, slot);
}

/// The pixel size at which the icon fills the slot, measured from its shape
/// at `base`.
pub fn fitted_size(
    context: &mut PaintContext<'_>,
    slot: PixelRect,
    glyph: Glyph,
    base: f32,
) -> Option<f32> {
    let measured = context.cache.glyph(context.fonts, glyph.face, glyph.id, base)?;
    let (width, height) = (measured.width as f32, measured.height as f32);
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    let room_x = slot.width as f32 * FILL / width;
    let room_y = slot.height as f32 * FILL / height;
    let scaled = base * room_x.min(room_y);
    Some((scaled * STEP).round().max(1.0) / STEP)
}

/// The pen that puts a bitmap in the middle of a slot.
pub fn centered_pen(slot: PixelRect, bitmap: &GlyphBitmap) -> (i32, i32) {
    let spare_x = (i64::from(slot.width) - i64::from(bitmap.width)).max(0) / 2;
    let spare_y = (i64::from(slot.height) - i64::from(bitmap.height)).max(0) / 2;
    (
        slot.x + spare_x as i32 - bitmap.left,
        slot.y + spare_y as i32 + bitmap.top,
    )
}

#[cfg(test)]
include!("icon_fit.tests.rs");
