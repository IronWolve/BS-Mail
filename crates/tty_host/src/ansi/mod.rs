//! Frames as escape sequences: one cell style as an SGR sequence, and a whole
//! frame as the lines a terminal prints.

pub mod color_map;
pub mod paint;
pub mod sgr;

pub use color_map::map_color;
pub use paint::{PaintOptions, paint_frame};
pub use sgr::{RESET, sgr, sgr_at};
