//! The terminal host: writes frames into a terminal as escape sequences and,
//! later, turns terminal input into input events. This file only declares and
//! re-exports the modules; later batches add the rest of the module map.

pub mod ansi;
pub mod screen_size;

pub use ansi::{PaintOptions, paint_frame};
pub use screen_size::terminal_size;
