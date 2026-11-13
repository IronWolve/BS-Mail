//! The embedded faces, the cell size they give, and the glyph cache.

pub mod embedded;
pub mod fallback;
pub mod glyph_cache;
pub mod metrics;
pub mod rasterize;

pub use embedded::{FaceKind, FontError, FontSet};
pub use fallback::{Glyph, choose, is_icon};
pub use glyph_cache::GlyphCache;
pub use metrics::CellMetrics;
pub use rasterize::{BitmapContent, GlyphBitmap, rasterize};
