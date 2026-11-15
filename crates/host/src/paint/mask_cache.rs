//! The shape of the banner's text, kept from one frame to the next.
//!
//! Stamping the letters into a mask, and blurring that mask for the glow, is
//! the dear part of a frame that has a banner. Neither changes until the
//! title, the face, the text size, the scale or the place of the banner does,
//! so a frame that only moved a selection finds both ready. Colors are not
//! part of the shape: they are applied when the mask is painted, so a theme
//! change reuses the same masks. A change of text size or scale clears the
//! glyph cache, which holds this one, so nothing stale outlives one.

use std::collections::VecDeque;

use surface::ScaledText;

use crate::font::CellMetrics;

use super::grid::PixelRect;
use super::text_mask::TextMask;

/// How many shapes are kept. A banner has two (its icon and its name); the
/// rest is room for the places a resize drag passes through.
const CAPACITY: usize = 8;

/// Everything the shape of one scaled-text effect depends on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaskKey {
    text: String,
    bold: bool,
    italic: bool,
    scale: u32,
    /// Where the effect's cells are on the canvas, and the box the mask covers
    /// (the cells plus the room the glow needs).
    area: PixelRect,
    bounds: PixelRect,
    /// The cell, and the glyph size in sixty-fourths of a pixel.
    cell: (u32, u32, i32),
    size: u32,
}

impl MaskKey {
    pub fn new(
        effect: &ScaledText,
        scale: u32,
        (area, bounds): (PixelRect, PixelRect),
        metrics: &CellMetrics,
    ) -> Self {
        Self {
            text: effect.text.clone(),
            bold: effect.bold,
            italic: effect.italic,
            scale,
            area,
            bounds,
            cell: (metrics.width, metrics.height, metrics.baseline),
            size: (metrics.pixel_size.max(0.0) * 64.0).round() as u32,
        }
    }
}

struct Entry {
    key: MaskKey,
    mask: TextMask,
    blank: bool,
    /// The mask blurred for a glow of this radius, once one was asked for.
    blurred: Option<(u8, TextMask)>,
}

/// A mask, and its blur when a glow was asked for.
pub struct Shape<'a> {
    pub mask: &'a TextMask,
    pub glow: Option<&'a TextMask>,
}

#[derive(Default)]
pub struct MaskCache {
    entries: VecDeque<Entry>,
    builds: usize,
    blurs: usize,
}

impl MaskCache {
    pub fn contains(&self, key: &MaskKey) -> bool {
        self.entries.iter().any(|entry| entry.key == *key)
    }

    /// Keeps a freshly stamped mask, dropping the oldest when full.
    pub fn insert(&mut self, key: MaskKey, mask: TextMask) {
        if self.entries.len() >= CAPACITY {
            self.entries.pop_front();
        }
        self.builds += 1;
        let blank = mask.is_blank();
        self.entries.push_back(Entry { key, mask, blank, blurred: None });
    }

    /// The kept shape for `key`, with its blur for a glow of `radius` (made
    /// the first time it is asked for). `None` when nothing is kept for the
    /// key, or the shape has no ink at all.
    pub fn shape(&mut self, key: &MaskKey, radius: Option<u8>) -> Option<Shape<'_>> {
        let entry = self.entries.iter_mut().find(|entry| entry.key == *key)?;
        if entry.blank {
            return None;
        }
        if let Some(radius) = radius.filter(|radius| *radius > 0) {
            let stale = entry.blurred.as_ref().is_none_or(|(kept, _)| *kept != radius);
            if stale {
                self.blurs += 1;
                entry.blurred = Some((radius, entry.mask.blurred(radius)));
            }
        }
        let glow = radius
            .filter(|radius| *radius > 0)
            .and(entry.blurred.as_ref())
            .map(|(_, blurred)| blurred);
        Some(Shape { mask: &entry.mask, glow })
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// How many masks were stamped since the start, for a check that a frame
    /// with nothing new does no stamping.
    pub fn builds(&self) -> usize {
        self.builds
    }

    /// How many blurs were computed since the start.
    pub fn blurs(&self) -> usize {
        self.blurs
    }
}

#[cfg(test)]
include!("mask_cache.tests.rs");
