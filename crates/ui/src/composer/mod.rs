//! The message composer: a text buffer, how it wraps, and the form around
//! it. This file only declares and re-exports the modules.

pub mod cursor_moves;
pub mod editor_buffer;
pub mod graphemes;
pub mod history;
pub mod wrap;

pub use editor_buffer::{EditorBuffer, Pos};
