//! The cell size, from the regular face at a pixel size.
//!
//! Cell width is the face's advance, cell height the line height times the
//! configured spacing, both whole pixels, so every cell lands on the grid and
//! lines drawn as shapes meet exactly.

use super::embedded::FontSet;

/// The pixel box one cell occupies, and where the text sits in it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellMetrics {
    pub width: u32,
    pub height: u32,
    /// From the cell's top edge down to the text baseline.
    pub baseline: i32,
    /// The size glyphs are rasterized at.
    pub pixel_size: f32,
    /// From the cell's top edge down to the underline.
    pub underline: i32,
    /// Thickness of underline, strikethrough and drawn lines.
    pub stroke: u32,
}

impl CellMetrics {
    /// The cell for `pixel_size` pixels per em, with rows loosened or
    /// tightened by `line_spacing`.
    pub fn new(fonts: &FontSet, pixel_size: f32, line_spacing: f32) -> Self {
        let face = fonts.regular();
        let metrics = face.metrics(&[]).scale(pixel_size);
        let glyphs = face.glyph_metrics(&[]).scale(pixel_size);
        let advance = glyphs.advance_width(face.charmap().map('M'));
        let width = round_up(advance);
        let line = metrics.ascent + metrics.descent + metrics.leading;
        let height = round_up(line * line_spacing);
        // The spacing's extra pixels are split above and below the text.
        let extra = (height as f32 - line) / 2.0;
        let baseline = (metrics.ascent + extra).round() as i32;
        let stroke = round_up(metrics.stroke_size);
        let below = (-metrics.underline_offset).max(metrics.stroke_size);
        let underline = (baseline + below.round() as i32).min(height as i32 - stroke as i32);
        Self {
            width,
            height,
            baseline,
            pixel_size,
            underline,
            stroke,
        }
    }

    /// The pixel size for a point size at a display scale factor. A point is
    /// a seventy-second of an inch and the reference display has ninety-six
    /// pixels to the inch, so twelve points is sixteen pixels before scaling.
    pub fn pixel_size(points: u8, scale_factor: f64) -> f32 {
        const PIXELS_PER_POINT: f64 = 96.0 / 72.0;
        (f64::from(points) * PIXELS_PER_POINT * scale_factor).max(1.0) as f32
    }

    /// The pixels a `cols` x `rows` grid needs.
    pub fn grid_pixels(&self, cols: u16, rows: u16) -> (u32, u32) {
        (self.width * u32::from(cols), self.height * u32::from(rows))
    }

    /// The whole cells that fit in a pixel box.
    pub fn grid_cells(&self, width: u32, height: u32) -> (u16, u16) {
        let cols = (width / self.width.max(1)).min(u32::from(u16::MAX)) as u16;
        let rows = (height / self.height.max(1)).min(u32::from(u16::MAX)) as u16;
        (cols, rows)
    }
}

/// At least one pixel, and never a fraction: cells are whole pixels.
fn round_up(value: f32) -> u32 {
    if value.is_finite() {
        value.round().max(1.0) as u32
    } else {
        1
    }
}

#[cfg(test)]
include!("metrics.tests.rs");
