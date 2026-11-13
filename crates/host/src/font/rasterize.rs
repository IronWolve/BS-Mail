//! One glyph into one small bitmap: an alpha mask for text and icons, or a
//! color bitmap for emoji faces that carry one.

use swash::FontRef;
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::zeno::Format;

/// What a glyph bitmap's bytes mean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmapContent {
    /// One coverage byte per pixel; the cell's color is painted through it.
    Mask,
    /// Four bytes per pixel, red green blue alpha, with its own colors.
    Color,
}

/// A rasterized glyph, placed relative to the pen position on the baseline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphBitmap {
    /// Pixels right of the pen to the bitmap's left edge.
    pub left: i32,
    /// Pixels above the baseline to the bitmap's top edge.
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub content: BitmapContent,
    pub data: Vec<u8>,
}

impl GlyphBitmap {
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0 || self.data.is_empty()
    }

    /// Coverage of one pixel, 0 to 255, for a mask bitmap.
    pub fn coverage(&self, x: u32, y: u32) -> u8 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        let index = (y * self.width + x) as usize;
        match self.content {
            BitmapContent::Mask => self.data.get(index).copied().unwrap_or(0),
            BitmapContent::Color => self.data.get(index * 4 + 3).copied().unwrap_or(0),
        }
    }

    /// Red, green, blue and alpha of one pixel of a color bitmap.
    pub fn color(&self, x: u32, y: u32) -> [u8; 4] {
        if x >= self.width || y >= self.height || self.content != BitmapContent::Color {
            return [0, 0, 0, 0];
        }
        let index = ((y * self.width + x) * 4) as usize;
        match self.data.get(index..index + 4) {
            Some(bytes) => [bytes[0], bytes[1], bytes[2], bytes[3]],
            None => [0, 0, 0, 0],
        }
    }
}

/// Rasterizes `glyph` of `face` at `pixel_size`. Color sources come first, so
/// a face with color glyphs gives color and every other face gives a mask.
pub fn rasterize(
    context: &mut ScaleContext,
    face: FontRef<'_>,
    glyph: u16,
    pixel_size: f32,
) -> Option<GlyphBitmap> {
    let mut scaler = context.builder(face).size(pixel_size).hint(true).build();
    let image = Render::new(&[
        Source::ColorOutline(0),
        Source::ColorBitmap(StrikeWith::BestFit),
        Source::Outline,
    ])
    .format(Format::Alpha)
    .render(&mut scaler, glyph)?;
    let content = match image.content {
        swash::scale::image::Content::Color => BitmapContent::Color,
        _ => BitmapContent::Mask,
    };
    Some(GlyphBitmap {
        left: image.placement.left,
        top: image.placement.top,
        width: image.placement.width,
        height: image.placement.height,
        content,
        data: image.data,
    })
}

#[cfg(test)]
include!("rasterize.tests.rs");
