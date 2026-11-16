//! The questions and boxes the folder actions, the tags and the attachments
//! open over the mail: a name to type (a new folder, a new name for a folder), a
//! yes or no before a folder goes for good, the picker of tags, and the path box
//! that asks where to save an attachment (see `path_entry`). At most one is open;
//! it takes every key and click until it is answered or cancelled. This file only
//! declares and re-exports the modules.

pub mod confirm;
pub mod confirm_draw;
pub mod name;
pub mod name_draw;
pub mod tags;
pub mod tags_draw;

pub use crate::path_entry::PathPrompt;
pub use confirm::{ConfirmAction, ConfirmPrompt};
pub use name::{NameFor, NamePrompt};
pub use tags::{TagPicker, TagRow};

/// The box that is open.
#[derive(Debug, Clone)]
pub enum Ask {
    Name(NamePrompt),
    Confirm(ConfirmPrompt),
    Tag(TagPicker),
    /// The path box: where to save an attachment.
    Path(PathPrompt),
}
