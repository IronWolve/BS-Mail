//! Which folders a search looks in.

use model::{Folder, FolderId, FolderRole};

use crate::imap_session::decode_mailbox_name;

/// A folder to search: its id and the name the server knows it by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Target {
    pub id: FolderId,
    pub path: String,
}

/// The folders to search, Inbox first and the rest in the order of the
/// folder tree. The saved searches (Starred, Alerts) are not mailboxes and
/// are never searched. Without `folder:` terms that is every folder except Trash
/// and Spam; with them, every folder whose path or name contains each of
/// the texts (any letter case), the Trash and Spam folders included when
/// they match.
pub(super) fn choose(folders: &[Folder], filters: &[String]) -> Vec<Target> {
    let mut chosen: Vec<&Folder> = folders
        .iter()
        .filter(|folder| !folder.role.is_virtual())
        .filter(|folder| {
            if filters.is_empty() {
                !matches!(folder.role, FolderRole::Trash | FolderRole::Spam)
            } else {
                filters.iter().all(|text| mentions(folder, text))
            }
        })
        .collect();
    chosen.sort_by_key(|folder| folder.role != FolderRole::Inbox);
    chosen.into_iter().map(|folder| Target { id: folder.id, path: folder.path.clone() }).collect()
}

/// True when the folder's path (as the server writes it or as people read
/// it) or its name contains `text`, in any letter case.
fn mentions(folder: &Folder, text: &str) -> bool {
    let needle = text.to_lowercase();
    [folder.path.as_str(), &decode_mailbox_name(&folder.path), folder.name.as_str()]
        .iter()
        .any(|hay| hay.to_lowercase().contains(&needle))
}

#[cfg(test)]
include!("search_folders.tests.rs");
