//! The window host: a frame of cells becomes pixels, and native window
//! events become input events. The only crate with window code.

pub mod font;
pub mod input;
pub mod paint;
pub mod present;
pub mod snapshot;
pub mod window;

pub use font::{CellMetrics, FaceKind, FontError, FontSet, GlyphCache};
pub use paint::{Canvas, Grid, PaintContext, Palette, Rgb, paint_frame};
pub use snapshot::{SnapshotError, write_png};
pub use window::{WindowOptions, WindowState, run_window};
