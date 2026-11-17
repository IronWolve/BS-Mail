//! What the reading pane can do with the message on screen beyond showing it:
//! select and copy its text, find words in it, show its raw headers or
//! source, and hand a safe copy of its HTML to the browser. This file only
//! declares and re-exports the modules.

mod attachment_state;
mod draw;
mod find;
mod find_text;
mod link_answers;
mod link_cursor;
mod raw_view;
mod selection;
mod state;

pub use attachment_state::{AttachmentFile, AttachmentState, Fetch, Owner, Want};
pub use find::FindState;
pub use find_text::find_matches;
pub(crate) use link_answers::Basis;
pub use link_answers::LinkAnswers;
pub use link_cursor::LinkCursor;
pub use raw_view::RawView;
pub use selection::TextSelection;
pub use state::{HtmlPage, ReaderTools};

#[cfg(test)]
pub(crate) use state::fixture;
