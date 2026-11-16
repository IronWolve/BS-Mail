//! What the backend remembers about one account while it runs: the ids it
//! handed out and what they stand for on the server.
//!
//! Folder and message ids are small numbers the backend invents; commands
//! come back carrying them, and this table maps them to a mailbox name and a
//! UID. It lives in memory for now. A store can later sit behind the same
//! methods and hand out durable ids instead.

use std::collections::HashMap;

use model::{Folder, FolderId, FolderRole, MessageId, Uid};

/// How a folder's UIDVALIDITY compares with the one its message ids were
/// issued under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Validity {
    /// The same: the ids still mean what they meant.
    Same,
    /// Different: every UID the folder had is void.
    Changed,
    /// One side is missing, so nothing can be confirmed.
    Unknown,
}

#[derive(Debug, Default)]
pub struct SessionState {
    next_folder: i64,
    next_message: i64,
    folder_ids: HashMap<String, FolderId>,
    folders: Vec<Folder>,
    validity: HashMap<FolderId, u32>,
    /// The server's separator between a folder and the one inside it, as the
    /// last listing gave it; `None` for a flat namespace or before any listing.
    delimiter: Option<String>,
    messages: HashMap<(FolderId, Uid), MessageId>,
    keys: HashMap<MessageId, (FolderId, Uid)>,
}

impl SessionState {
    pub fn new() -> Self {
        Self::default()
    }

    /// The id for a mailbox name; the same name always gets the same id, also
    /// after the folder list is read again.
    pub fn folder_id(&mut self, path: &str) -> FolderId {
        if let Some(id) = self.folder_ids.get(path) {
            return *id;
        }
        self.next_folder += 1;
        let id = FolderId(self.next_folder);
        self.folder_ids.insert(path.to_owned(), id);
        id
    }

    /// Replaces the folder tree with a fresh listing.
    pub fn set_folders(&mut self, folders: Vec<Folder>) {
        self.folders = folders;
    }

    /// The folder with this id in the current listing.
    pub fn folder(&self, id: FolderId) -> Option<&Folder> {
        self.folders.iter().find(|folder| folder.id == id)
    }

    /// The server's name for a folder in the current listing. A saved search
    /// (Starred, Alerts) is not a mailbox and has none, so no command ever
    /// carries its name to the server.
    pub fn folder_path(&self, id: FolderId) -> Option<&str> {
        self.folder(id).filter(|folder| !folder.role.is_virtual()).map(|folder| folder.path.as_str())
    }

    /// The separator the server uses between a folder and the one inside it.
    pub fn delimiter(&self) -> Option<&str> {
        self.delimiter.as_deref()
    }

    pub fn set_delimiter(&mut self, delimiter: Option<String>) {
        self.delimiter = delimiter;
    }

    /// A folder was renamed on the server: it and every folder inside it keep
    /// their ids under their new names, so what the screen holds (the open
    /// folder, the messages shown) stays valid.
    pub fn rename_folder_paths(&mut self, from: &str, to: &str) {
        let separator = self.delimiter.clone().unwrap_or_default();
        let moved: Vec<(String, FolderId)> = self
            .folder_ids
            .iter()
            .filter(|(path, _)| {
                path.as_str() == from
                    || (!separator.is_empty() && path.strip_prefix(from).is_some_and(|rest| rest.starts_with(separator.as_str())))
            })
            .map(|(path, id)| (path.clone(), *id))
            .collect();
        for (path, id) in moved {
            self.folder_ids.remove(&path);
            self.folder_ids.insert(format!("{to}{}", &path[from.len()..]), id);
        }
    }

    /// A folder was deleted on the server: nothing known about its messages
    /// stays, so a folder made later under the same name starts clean.
    pub fn forget_folder(&mut self, id: FolderId) {
        self.validity.remove(&id);
        self.forget_messages(id);
    }

    /// The current listing.
    pub fn folders(&self) -> &[Folder] {
        &self.folders
    }

    /// The folder that holds deleted mail, when the account has one.
    pub fn trash_folder(&self) -> Option<&Folder> {
        self.folders
            .iter()
            .find(|folder| folder.role == FolderRole::Trash)
    }

    /// Stores new counts for a folder in the listing. True when they differ
    /// from the ones held.
    pub fn set_counts(&mut self, id: FolderId, unread: u32, total: u32) -> bool {
        let Some(folder) = self.folders.iter_mut().find(|folder| folder.id == id) else {
            return false;
        };
        let moved = folder.unread != unread || folder.total != total;
        folder.unread = unread;
        folder.total = total;
        moved
    }

    /// The id for a message; the same folder and UID always get the same id.
    pub fn message_id(&mut self, folder: FolderId, uid: Uid) -> MessageId {
        if let Some(id) = self.messages.get(&(folder, uid)) {
            return *id;
        }
        self.next_message += 1;
        let id = MessageId(self.next_message);
        self.messages.insert((folder, uid), id);
        self.keys.insert(id, (folder, uid));
        id
    }

    /// Where a message lives on the server.
    pub fn message_key(&self, id: MessageId) -> Option<(FolderId, Uid)> {
        self.keys.get(&id).copied()
    }

    /// Stops a message id from resolving, for a message that left its folder.
    pub fn forget_message(&mut self, id: MessageId) {
        if let Some(key) = self.keys.remove(&id) {
            self.messages.remove(&key);
        }
    }

    /// Checks the UIDVALIDITY a server reports for a folder just opened for a
    /// change against the one its ids were issued under. Anything but `Same`
    /// voids the folder's ids, so no later command can act on them.
    pub fn check_validity(&mut self, folder: FolderId, seen: Option<u32>) -> Validity {
        let known = self.validity.get(&folder).copied();
        match (known, seen) {
            (Some(before), Some(now)) if before == now => Validity::Same,
            (Some(_), Some(now)) => {
                self.validity.insert(folder, now);
                self.forget_messages(folder);
                Validity::Changed
            }
            _ => {
                self.forget_messages(folder);
                Validity::Unknown
            }
        }
    }

    /// Records the folder's UIDVALIDITY. When it differs from the one seen
    /// before, every UID the folder had is void: its message ids are
    /// forgotten and `true` is returned.
    pub fn note_validity(&mut self, folder: FolderId, validity: Option<u32>) -> bool {
        let Some(validity) = validity else {
            return false;
        };
        let changed = self
            .validity
            .insert(folder, validity)
            .is_some_and(|before| before != validity);
        if changed {
            self.forget_messages(folder);
        }
        changed
    }

    fn forget_messages(&mut self, folder: FolderId) {
        self.messages.retain(|(owner, _), _| *owner != folder);
        self.keys.retain(|_, (owner, _)| *owner != folder);
    }
}

#[cfg(test)]
include!("session_state.tests.rs");
