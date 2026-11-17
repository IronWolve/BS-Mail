//! What the picker holds: what the chosen folder is for, the messages being
//! moved when it is for a move, the text typed to narrow the folders down, and
//! which one is highlighted.

use model::{Folder, FolderId, FolderRole, MessageId};
use unicode_segmentation::UnicodeSegmentation;

/// What the folder chosen in the picker is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    /// Move the messages of the picker there.
    Move,
    /// Open it: go to the folder. Every folder of the account is a place to
    /// go to, the one open and the virtual ones included.
    Open,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovePicker {
    pub purpose: Purpose,
    /// The messages that will move; none when the picker is for going to a
    /// folder.
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
            purpose: Purpose::Move,
            targets,
            from,
            filter: String::new(),
            cursor: 0,
        }
    }

    /// A picker for going to a folder: it holds no messages and leaves no
    /// folder out.
    pub fn for_opening() -> Self {
        Self {
            purpose: Purpose::Open,
            targets: Vec::new(),
            from: None,
            filter: String::new(),
            cursor: 0,
        }
    }

    /// The folders that can be chosen, in the sidebar's order. To move
    /// messages: not the one they are in, and not the virtual ones that are
    /// queries and not mailboxes. To go to a folder: every one. Then only
    /// those whose name or path holds the typed text.
    pub fn candidates<'a>(&self, folders: &'a [Folder]) -> Vec<&'a Folder> {
        let needle = self.filter.trim().to_lowercase();
        folders
            .iter()
            .filter(|folder| self.purpose == Purpose::Open || self.takes_messages(folder))
            .filter(|folder| {
                needle.is_empty()
                    || folder.name.to_lowercase().contains(&needle)
                    || folder.path.to_lowercase().contains(&needle)
            })
            .collect()
    }

    /// Whether `folder` is a place to move the messages to.
    fn takes_messages(&self, folder: &Folder) -> bool {
        Some(folder.id) != self.from && !matches!(folder.role, FolderRole::Starred | FolderRole::Alerts)
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

    /// Removes the last character as the user sees it: a letter with its
    /// accent, or an emoji with its modifier, goes whole.
    pub fn backspace(&mut self) {
        if let Some((start, _)) = self.filter.grapheme_indices(true).next_back() {
            self.filter.truncate(start);
        }
        self.cursor = 0;
    }
}

#[cfg(test)]
include!("state.tests.rs");
