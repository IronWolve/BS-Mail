//! What may be done to a folder, worded once for the screens and the engine
//! (the real backend and the mock): which folders cannot be renamed, deleted
//! or emptied. Each answer is `None` when the action is allowed, else the
//! plain sentence for the footer.

use crate::folder::{Folder, FolderRole};
use crate::ids::FolderId;

/// Why a folder with a role cannot be renamed or deleted, or `None` for a
/// plain folder. `done` is the past tense of what was asked ("renamed",
/// "deleted").
pub fn protected(folder: &Folder, done: &str) -> Option<String> {
    match folder.role {
        FolderRole::Plain => None,
        FolderRole::Starred | FolderRole::Alerts => {
            Some(format!("{} is a saved search, not a folder; nothing was {done}", folder.name))
        }
        _ => Some(format!("{} is a special folder the program relies on; it cannot be {done}", folder.name)),
    }
}

/// Why a folder that holds other folders cannot be deleted, or `None`.
pub fn holds_folders(folders: &[Folder], id: FolderId, name: &str) -> Option<String> {
    folders
        .iter()
        .any(|folder| folder.parent == Some(id))
        .then(|| format!("{name} holds other folders; delete or move those first, nothing was deleted"))
}

/// Why a folder cannot be emptied, or `None` for the Trash and Spam folders,
/// the only two that can.
pub fn not_emptiable(folder: &Folder) -> Option<String> {
    match folder.role {
        FolderRole::Trash | FolderRole::Spam => None,
        FolderRole::Starred | FolderRole::Alerts => {
            Some(format!("{} is a saved search, not a folder; nothing was changed", folder.name))
        }
        _ => Some(format!("only the Trash and Spam folders can be emptied; {} is neither", folder.name)),
    }
}

/// Why a saved search (Starred, Alerts) cannot take a change to a folder, or
/// `None` for a mailbox.
pub fn saved_search_refusal(folder: &Folder) -> Option<String> {
    folder.role.is_virtual().then(|| format!("{} is a saved search, not a folder; nothing was changed", folder.name))
}

#[cfg(test)]
include!("folder_rules.tests.rs");
