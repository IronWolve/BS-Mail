//! Text at a whole multiple of the cell size: the banner title.
//!
//! The run is stamped into a mask, the glow is laid down from that same
//! shape, and the letters are then colored pixel by pixel, which is what
//! makes the gradient smooth rather than one color per letter.

use surface::{ScaledText, TextFill};
use unicode_width::UnicodeWidthStr;

use crate::font::{FaceKind, choose, is_icon};

use super::canvas::Canvas;
use super::colors::{Ink, Rgb};
use super::context::PaintContext;
use super::glow::draw_glow;
use super::grid::PixelRect;
use super::icon_fit::{centered_pen, fitted_size};
use super::text_mask::TextMask;

/// Draws one scaled-text effect.
pub fn draw_scaled_text(canvas: &mut Canvas, context: &mut PaintContext<'_>, effect: &ScaledText) {
    let scale = u32::from(effect.scale.max(1));
    let rect = context.grid.area(effect.area);
    let margin = effect.glow.map_or(0, |glow| u32::from(glow.radius) * scale);
    let mask = build_mask(context, effect, rect.grow(margin), scale);
    if mask.is_blank() {
        return;
    }
    if let Some(glow) = effect.glow {
        let color = context.palette.resolve(glow.color, Ink::Foreground);
        let radius = u8::try_from(u32::from(glow.radius) * scale).unwrap_or(u8::MAX);
        draw_glow(canvas, &mask, color, radius);
    }
    let stops = fill_stops(context, &effect.fill);
    paint_mask(canvas, &mask, rect, &stops);
}

/// Stamps every character of the run into a mask at the scaled size.
fn build_mask(
    context: &mut PaintContext<'_>,
    effect: &ScaledText,
    bounds: PixelRect,
    scale: u32,
) -> TextMask {
    let mut mask = TextMask::new(bounds);
    let metrics = context.grid.metrics;
    let size = metrics.pixel_size * scale as f32;
    let top = context.grid.y(effect.area.y);
    let baseline = top + metrics.baseline as i32 * scale as i32;
    let height = metrics.height * scale;
    let face = FaceKind::text(effect.bold, effect.italic);
    let mut pen = context.grid.x(effect.area.x);
    // A run of one character, such as the banner's icon, is given the whole
    // area the effect was handed; a longer run shares it out per character.
    let single = effect.text.chars().count() == 1;
    let whole = u32::from(effect.area.width) * metrics.width;
    for ch in effect.text.chars() {
        let advance = metrics.width * scale * cell_width(ch);
        let width = if single { whole } else { advance };
        let slot = PixelRect::new(pen, top, width, height);
        stamp(&mut mask, context, (ch, face, size), (slot, baseline));
        pen += advance as i32;
    }
    mask
}

/// One character into the mask: an icon fills its slot, a letter sits on the
/// baseline.
fn stamp(
    mask: &mut TextMask,
    context: &mut PaintContext<'_>,
    (ch, face, size): (char, FaceKind, f32),
    (slot, baseline): (PixelRect, i32),
) {
    let Some(glyph) = choose(context.fonts, face, ch) else {
        return;
    };
    let (size, pen) = if is_icon(ch) {
        let Some(fitted) = fitted_size(context, slot, glyph, size) else {
            return;
        };
        (fitted, None)
    } else {
        (size, Some((slot.x, baseline)))
    };
    let Some(bitmap) = context.cache.glyph(context.fonts, glyph.face, glyph.id, size) else {
        return;
    };
    let pen = pen.unwrap_or_else(|| centered_pen(slot, bitmap));
    mask.stamp(bitmap, pen);
}

/// Cells one character takes at normal size.
fn cell_width(ch: char) -> u32 {
    UnicodeWidthStr::width(ch.to_string().as_str()).max(1) as u32
}

/// The colors a fill blends through, left to right.
fn fill_stops(context: &PaintContext<'_>, fill: &TextFill) -> Vec<Rgb> {
    match fill {
        TextFill::Solid(color) => vec![context.palette.resolve(*color, Ink::Foreground)],
        TextFill::Gradient(colors) => colors
            .iter()
            .map(|color| context.palette.resolve(*color, Ink::Foreground))
            .collect(),
    }
}

/// Colors the mask across `span`, so the gradient follows the text's own
/// width rather than the glow's wider box.
fn paint_mask(canvas: &mut Canvas, mask: &TextMask, span: PixelRect, stops: &[Rgb]) {
    if stops.is_empty() {
        return;
    }
    let rect = mask.rect;
    for y in rect.y..rect.bottom() {
        for x in rect.x..rect.right() {
            let coverage = mask.coverage(x, y);
            if coverage == 0 {
                continue;
            }
            let across = (x - span.x).clamp(0, span.width.max(1) as i32) as f32;
            let color = blend_stops(stops, across / span.width.max(1) as f32);
            canvas.blend(x, y, color, coverage);
        }
    }
}

/// The color at `position`, 0 at the first stop and 1 at the last.
fn blend_stops(stops: &[Rgb], position: f32) -> Rgb {
    if stops.len() == 1 {
        return stops[0];
    }
    let spans = (stops.len() - 1) as f32;
    let place = (position.clamp(0.0, 1.0) * spans).min(spans - f32::EPSILON);
    let index = place as usize;
    let amount = ((place - index as f32) * 255.0).round() as u8;
    stops[index + 1].mix(stops[index], amount)
}

#[cfg(test)]
include!("scaled_text.tests.rs");
