//! The tag picker: the tags known on the account, a line to type in, and a
//! row for a tag that is new. What is typed narrows the known tags down; a
//! typed word that is no known tag is offered as a new one when it could be
//! one, and says why when it could not.

use model::{MessageId, check_tag};
use unicode_segmentation::UnicodeSegmentation;

/// How many of the messages a tag is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Held {
    All,
    Some,
    NoneOfThem,
}

/// One row of the picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagRow {
    pub tag: String,
    pub held: Held,
    /// A tag that is not known yet: typed, and fit to be one.
    pub is_new: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagPicker {
    /// The messages the tag is set on or cleared from.
    pub targets: Vec<MessageId>,
    /// The tags known on the account, by name, with how the targets hold each.
    known: Vec<(String, Held)>,
    pub filter: String,
    /// Which row is highlighted.
    pub cursor: usize,
}

impl TagPicker {
    pub fn new(targets: Vec<MessageId>, mut known: Vec<(String, Held)>) -> Self {
        known.sort_by_key(|(tag, _)| tag.to_lowercase());
        Self { targets, known, filter: String::new(), cursor: 0 }
    }

    /// The rows now: the known tags that hold the typed text, then a row for
    /// the typed word when it is new and could be a tag.
    pub fn rows(&self) -> Vec<TagRow> {
        let needle = self.filter.trim().to_lowercase();
        let mut rows: Vec<TagRow> = self
            .known
            .iter()
            .filter(|(tag, _)| needle.is_empty() || tag.to_lowercase().contains(&needle))
            .map(|(tag, held)| TagRow { tag: tag.clone(), held: *held, is_new: false })
            .collect();
        if let Some(word) = self.new_word() {
            rows.push(TagRow { tag: word, held: Held::NoneOfThem, is_new: true });
        }
        rows
    }

    /// The typed word, when it is not a known tag (in any letter case) and is
    /// fit to be one.
    fn new_word(&self) -> Option<String> {
        let word = self.filter.trim();
        let known = self.known.iter().any(|(tag, _)| tag.eq_ignore_ascii_case(word));
        if word.is_empty() || known { None } else { check_tag(word).ok() }
    }

    /// Why the typed text cannot be a new tag, when something is typed that
    /// matches no known tag and is not fit to be one.
    pub fn problem(&self) -> Option<String> {
        let word = self.filter.trim();
        let any_match = self.rows().iter().any(|row| !row.is_new);
        if word.is_empty() || any_match || self.new_word().is_some() { None } else { check_tag(word).err() }
    }

    /// The highlighted row.
    pub fn chosen(&self) -> Option<TagRow> {
        self.rows().into_iter().nth(self.cursor)
    }

    /// Moves the highlight, staying on the list.
    pub fn step(&mut self, down: bool) {
        let shown = self.rows().len();
        if shown == 0 {
            self.cursor = 0;
        } else if down {
            self.cursor = (self.cursor + 1).min(shown - 1);
        } else {
            self.cursor = self.cursor.saturating_sub(1);
        }
    }

    /// Adds typed text; the highlight goes back to the top.
    pub fn type_text(&mut self, text: &str) {
        self.filter.extend(text.chars().filter(|ch| !ch.is_control()));
        self.cursor = 0;
    }

    /// Removes the last character as the user sees it.
    pub fn backspace(&mut self) {
        if let Some((start, _)) = self.filter.grapheme_indices(true).next_back() {
            self.filter.truncate(start);
        }
        self.cursor = 0;
    }
}

#[cfg(test)]
include!("tags.tests.rs");
