//! What every drawing call needs: the faces, the glyph cache, the cell
//! geometry and what the base colors are.

use crate::font::{FontSet, GlyphCache};

use super::colors::Palette;
use super::grid::Grid;

pub struct PaintContext<'a> {
    pub fonts: &'a FontSet,
    pub cache: &'a mut GlyphCache,
    pub grid: Grid,
    pub palette: Palette,
    /// The display's scale factor, which the line weights are multiplied by.
    pub scale: f64,
}

impl<'a> PaintContext<'a> {
    /// A context at scale factor 1.
    pub fn new(fonts: &'a FontSet, cache: &'a mut GlyphCache, grid: Grid, palette: Palette) -> Self {
        Self {
            fonts,
            cache,
            grid,
            palette,
            scale: 1.0,
        }
    }

    /// The same context on a display with this scale factor.
    pub fn with_scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    /// Pixels for a line weight given at scale factor 1: never less than one.
    pub fn line_pixels(&self, weight: u8) -> u32 {
        (f64::from(weight.max(1)) * self.scale.max(0.0)).round().max(1.0) as u32
    }

    /// The thickness of the lines drawn for frame borders. Never thinner than
    /// the font's own stroke, so text that is larger than the reference size
    /// keeps the borders it had; a heavier weight than that thickens them.
    pub fn frame_stroke(&self, weight: u8) -> u32 {
        self.line_pixels(weight).max(self.grid.metrics.stroke)
    }

    /// The size glyphs are rasterized at for normal text.
    pub fn pixel_size(&self) -> f32 {
        self.grid.metrics.pixel_size
    }
}
