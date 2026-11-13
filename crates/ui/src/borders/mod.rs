//! Border glyph sets and the helpers that pick the right piece where lines
//! meet. This file only declares and re-exports them.

pub mod border_set;
pub mod frame_lines;
pub mod line_map;

pub use border_set::{BorderSet, DOUBLE, PLAIN, ROUNDED, THICK};
pub use frame_lines::{Neighbors, junction_piece};
pub use line_map::LineMap;
