//! The screen lines of one buffer, kept between calls.
//!
//! Wrapping cuts the whole text into graphemes, which is slow for a long
//! body, and the composer asks for it on every paint and on every vertical
//! step. The cache holds the answer for the last width asked and the text
//! revision it was made for, so asking again costs nothing until the text
//! changes or the pane is resized.

use std::sync::{Arc, Mutex, PoisonError};

use super::wrap::{VisualLine, wrap};

/// The last wrapping of a buffer. Behind a lock so a paint, which only holds
/// the buffer by shared reference, can fill it.
#[derive(Debug, Default)]
pub(super) struct WrapCache(Mutex<Option<Cached>>);

#[derive(Debug)]
struct Cached {
    revision: u64,
    width: usize,
    visual: Arc<[VisualLine]>,
}

impl Clone for WrapCache {
    /// A copy of a buffer starts with nothing kept and fills itself on first
    /// use.
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl WrapCache {
    /// The screen lines of `lines` at `width` cells, wrapped only when the
    /// text `revision` or the width differs from the last ask.
    pub(super) fn visual(&self, revision: u64, lines: &[String], width: usize) -> Arc<[VisualLine]> {
        let width = width.max(1);
        let mut kept = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(cached) = kept.as_ref().filter(|cached| cached.revision == revision && cached.width == width) {
            return Arc::clone(&cached.visual);
        }
        let visual: Arc<[VisualLine]> = wrap(lines, width).into();
        *kept = Some(Cached { revision, width, visual: Arc::clone(&visual) });
        visual
    }
}

#[cfg(test)]
include!("wrap_cache.tests.rs");
