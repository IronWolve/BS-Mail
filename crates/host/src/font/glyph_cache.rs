//! Rasterized glyphs kept by face, glyph and pixel size.
//!
//! A full screen repeats a few dozen shapes thousands of times, so every
//! glyph is rasterized once. A font-size or scale-factor change clears the
//! cache; nothing else invalidates an entry. The banner's text shapes are
//! kept here too, because they go stale at the same moments.

use std::collections::HashMap;

use swash::scale::ScaleContext;

use crate::paint::mask_cache::MaskCache;

use super::embedded::{FaceKind, FontSet};
use super::rasterize::{GlyphBitmap, rasterize};

/// One rasterized shape: a face, a glyph of it and the size it was drawn at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    face: FaceKind,
    glyph: u16,
    /// The pixel size in sixty-fourths, so sizes compare exactly.
    size: u32,
}

#[derive(Default)]
pub struct GlyphCache {
    context: ScaleContext,
    /// `None` records a glyph that rasterized to nothing, so it is not tried
    /// again every frame.
    entries: HashMap<Key, Option<GlyphBitmap>>,
    /// The banner's text shapes, which depend on the same sizes and are
    /// dropped with the glyphs.
    pub masks: MaskCache,
}

impl GlyphCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// The bitmap for one glyph, rasterizing it the first time it is asked for.
    pub fn glyph(
        &mut self,
        fonts: &FontSet,
        face: FaceKind,
        glyph: u16,
        pixel_size: f32,
    ) -> Option<&GlyphBitmap> {
        let key = Key {
            face,
            glyph,
            size: (pixel_size.max(0.0) * 64.0).round() as u32,
        };
        let entry = self.entries.entry(key).or_insert_with(|| {
            let font = fonts.face(face)?;
            rasterize(&mut self.context, font, glyph, pixel_size)
                .filter(|bitmap| !bitmap.is_empty())
        });
        entry.as_ref()
    }

    /// Drops every entry, after a font-size or scale-factor change.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.masks.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
include!("glyph_cache.tests.rs");
