//! Frames as escape sequences: one cell style as an SGR sequence, and a whole
//! frame as the lines a terminal prints.

pub mod paint;
pub mod sgr;

pub use paint::{PaintOptions, paint_frame};
pub use sgr::{RESET, sgr};
