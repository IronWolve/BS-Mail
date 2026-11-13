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
}

impl<'a> PaintContext<'a> {
    pub fn new(fonts: &'a FontSet, cache: &'a mut GlyphCache, grid: Grid, palette: Palette) -> Self {
        Self {
            fonts,
            cache,
            grid,
            palette,
        }
    }

    /// The size glyphs are rasterized at for normal text.
    pub fn pixel_size(&self) -> f32 {
        self.grid.metrics.pixel_size
    }
}
