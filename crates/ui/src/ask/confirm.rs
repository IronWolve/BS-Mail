//! A yes or no before a folder goes for good, or before an attachment that can
//! carry active content is opened: what is asked, what is said under it and what
//! the two buttons read. Focus starts on the safe button.

use model::{AttachmentInfo, Folder, FolderId, FolderRole, MessageId};

use crate::delete_prompt::Choice;
use crate::format::truncate;

/// What a yes does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    /// Remove the folder and every message in it.
    DeleteFolder(FolderId),
    /// Remove every message of the Trash or Spam folder.
    EmptyFolder(FolderId),
    /// Save the attachment and open it with the system.
    OpenAttachment { message: MessageId, attachment: AttachmentInfo },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmPrompt {
    pub action: ConfirmAction,
    pub title: &'static str,
    pub question: String,
    pub warning: &'static str,
    /// The first button, which does it, and the second, which keeps things as
    /// they are; each with a blank on both sides.
    pub yes: &'static str,
    pub no: &'static str,
    pub focus: Choice,
    /// What the footer says once the server has done it.
    pub done: String,
}

const WARNING: &str = "This cannot be undone.";
const ACTIVE_CONTENT_WARNING: &str = "PDF, Office and RTF files can carry active content.";
/// The most of an attachment's name that the question shows, so that the line fits the box.
const NAME_CELLS: usize = 34;

fn messages(count: u32) -> String {
    if count == 1 { "1 message".to_owned() } else { format!("{count} messages") }
}

impl ConfirmPrompt {
    /// The question before `folder` is deleted, which says how many messages
    /// go with it.
    pub fn delete_folder(folder: &Folder) -> Self {
        let question = match folder.total {
            0 => format!("Delete the empty folder {}?", folder.name),
            count => format!("Delete the folder {} and its {} for good?", folder.name, messages(count)),
        };
        Self {
            action: ConfirmAction::DeleteFolder(folder.id),
            title: " Delete folder ",
            question,
            warning: WARNING,
            yes: " Delete folder ",
            no: " Keep it ",
            focus: Choice::Keep,
            done: format!("Deleted the folder {}", folder.name),
        }
    }

    /// The question before every message of `folder` (a Trash or Spam folder)
    /// is deleted, which says how many.
    pub fn empty_folder(folder: &Folder) -> Self {
        let place = if folder.role == FolderRole::Spam { "the Spam folder" } else { "the Trash" };
        let question = match folder.total {
            1 => format!("Delete the 1 message in {place} for good?"),
            count => format!("Delete all {count} messages in {place} for good?"),
        };
        Self {
            action: ConfirmAction::EmptyFolder(folder.id),
            title: if folder.role == FolderRole::Spam { " Empty Spam " } else { " Empty Trash " },
            question,
            warning: WARNING,
            yes: " Delete all ",
            no: " Keep them ",
            focus: Choice::Keep,
            done: format!("Emptied {place}"),
        }
    }

    /// The question before the system opens `attachment` of `message`: a kind of
    /// file that can carry active content (see `active_content`). A no, the
    /// default, leaves it unopened (it can still be saved).
    pub fn open_attachment(message: MessageId, attachment: &AttachmentInfo) -> Self {
        let name = truncate(&attachment.name, NAME_CELLS, "...");
        Self {
            action: ConfirmAction::OpenAttachment { message, attachment: attachment.clone() },
            title: " Open attachment ",
            question: format!("Open {name} with the system viewer?"),
            warning: ACTIVE_CONTENT_WARNING,
            yes: " Open it ",
            no: " Do not open ",
            focus: Choice::Keep,
            done: String::new(),
        }
    }

    /// Moves focus to the other button.
    pub fn toggle(&mut self) {
        self.focus = match self.focus {
            Choice::Delete => Choice::Keep,
            Choice::Keep => Choice::Delete,
        };
    }
}

#[cfg(test)]
include!("confirm.tests.rs");
