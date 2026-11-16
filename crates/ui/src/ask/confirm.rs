//! A yes or no before a folder goes for good: which folder, what is said
//! and what the two buttons read. Focus starts on the safe button.

use model::{Folder, FolderId, FolderRole};

use crate::delete_prompt::Choice;

/// What a yes does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    /// Remove the folder and every message in it.
    DeleteFolder(FolderId),
    /// Remove every message of the Trash or Spam folder.
    EmptyFolder(FolderId),
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
