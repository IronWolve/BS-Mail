//! The message composer: a text buffer, how it wraps, and the form around
//! it. This file only declares and re-exports the modules.

pub mod address_book;
pub mod address_list;
pub mod buttons;
pub mod cursor_moves;
pub mod draw;
pub mod editor_buffer;
pub mod form;
mod form_sender;
pub mod graphemes;
pub mod history;
pub mod input;
pub mod mailto;
pub mod mouse;
pub(crate) mod pointer_moves;
pub mod popup;
pub mod reply;
pub mod sender;
mod sender_keys;
pub mod signature;
pub mod suggest;
pub mod wrap;
mod wrap_cache;

pub use address_book::AddressBook;
pub use editor_buffer::{EditorBuffer, Pos};
pub use form::{ComposeForm, Field, Progress};
pub use mailto::{MailtoDraft, MailtoError, looks_like_address, parse_mailto};
pub use reply::{Draft, Original};
pub use sender::Sender;
