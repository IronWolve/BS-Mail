//! What the picker holds: the messages being moved, the text typed to narrow
//! the folders down, and which one is highlighted.

use model::{Folder, FolderId, FolderRole, MessageId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovePicker {
    /// The messages that will move.
    pub targets: Vec<MessageId>,
    /// The folder they are all in now, which is not a place to move them to;
    /// `None` when they come from several folders, so every folder is one.
    pub from: Option<FolderId>,
    pub filter: String,
    /// Which of the shown folders is highlighted.
    pub cursor: usize,
}

impl MovePicker {
    pub fn new(targets: Vec<MessageId>, from: Option<FolderId>) -> Self {
        Self {
            targets,
            from,
            filter: String::new(),
            cursor: 0,
        }
    }

    /// The folders that can be chosen, in the sidebar's order: not the one
    /// the messages are in, and not the virtual ones that are queries and not
    /// mailboxes, then only those whose name or path holds the typed text.
    pub fn candidates<'a>(&self, folders: &'a [Folder]) -> Vec<&'a Folder> {
        let needle = self.filter.trim().to_lowercase();
        folders
            .iter()
            .filter(|folder| Some(folder.id) != self.from)
            .filter(|folder| !matches!(folder.role, FolderRole::Starred | FolderRole::Alerts))
            .filter(|folder| {
                needle.is_empty()
                    || folder.name.to_lowercase().contains(&needle)
                    || folder.path.to_lowercase().contains(&needle)
            })
            .collect()
    }

    /// The highlighted folder, if any folder is shown.
    pub fn chosen<'a>(&self, folders: &'a [Folder]) -> Option<&'a Folder> {
        self.candidates(folders).get(self.cursor).copied()
    }

    /// Moves the highlight, staying on the list.
    pub fn step(&mut self, down: bool, shown: usize) {
        if shown == 0 {
            self.cursor = 0;
        } else if down {
            self.cursor = (self.cursor + 1).min(shown - 1);
        } else {
            self.cursor = self.cursor.saturating_sub(1);
        }
    }

    /// Adds typed text; the highlight goes back to the top, since the list
    /// just changed under it.
    pub fn type_text(&mut self, text: &str) {
        self.filter.extend(text.chars().filter(|ch| !ch.is_control()));
        self.cursor = 0;
    }

    pub fn backspace(&mut self) {
        self.filter.pop();
        self.cursor = 0;
    }
}

#[cfg(test)]
include!("state.tests.rs");
