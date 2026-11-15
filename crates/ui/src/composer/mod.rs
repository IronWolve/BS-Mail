//! The message composer: a text buffer, how it wraps, and the form around
//! it. This file only declares and re-exports the modules.

pub mod address_list;
pub mod cursor_moves;
pub mod draw;
pub mod editor_buffer;
pub mod form;
pub mod graphemes;
pub mod history;
pub mod input;
pub mod reply;
pub mod wrap;
mod wrap_cache;

pub use editor_buffer::{EditorBuffer, Pos};
pub use form::{ComposeForm, Field, Progress};
pub use reply::{Draft, Original};
