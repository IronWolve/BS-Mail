//! What the backend remembers about one account while it runs: the ids it
//! handed out and what they stand for on the server.
//!
//! Folder and message ids are small numbers the backend invents; commands
//! come back carrying them, and this table maps them to a mailbox name and a
//! UID. It lives in memory for now. A store can later sit behind the same
//! methods and hand out durable ids instead.

use std::collections::HashMap;

use model::{Folder, FolderId, MessageId, Uid};

#[derive(Debug, Default)]
pub struct SessionState {
    next_folder: i64,
    next_message: i64,
    folder_ids: HashMap<String, FolderId>,
    folders: Vec<Folder>,
    validity: HashMap<FolderId, u32>,
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

    /// The server's name for a folder in the current listing.
    pub fn folder_path(&self, id: FolderId) -> Option<&str> {
        self.folders
            .iter()
            .find(|folder| folder.id == id)
            .map(|folder| folder.path.as_str())
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
