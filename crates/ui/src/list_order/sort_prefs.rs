//! The sort chosen for each kind of folder, kept between runs. A folder's own
//! name does not matter, only what it is for: every plain folder shares one
//! choice, as do the Inbox, Sent and the others. The state file keeps each as
//! text: `size descending`.

use std::collections::BTreeMap;

use model::FolderRole;

use super::sort::{Sort, SortColumn};

/// The kinds of folder a sort is kept for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RoleKey {
    Inbox,
    Sent,
    Drafts,
    Trash,
    Spam,
    Archive,
    Plain,
}

impl RoleKey {
    pub const ALL: [RoleKey; 7] =
        [Self::Inbox, Self::Sent, Self::Drafts, Self::Trash, Self::Spam, Self::Archive, Self::Plain];

    /// The kind a folder of `role` belongs to. The saved searches that sit
    /// among the folders are plain.
    pub fn of(role: FolderRole) -> Self {
        match role {
            FolderRole::Inbox => Self::Inbox,
            FolderRole::Sent => Self::Sent,
            FolderRole::Drafts => Self::Drafts,
            FolderRole::Trash => Self::Trash,
            FolderRole::Spam => Self::Spam,
            FolderRole::Archive => Self::Archive,
            FolderRole::Starred | FolderRole::Alerts | FolderRole::Plain => Self::Plain,
        }
    }

    /// The key the state file uses.
    pub fn name(self) -> &'static str {
        match self {
            Self::Inbox => "inbox",
            Self::Sent => "sent",
            Self::Drafts => "drafts",
            Self::Trash => "trash",
            Self::Spam => "spam",
            Self::Archive => "archive",
            Self::Plain => "plain",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.trim().to_ascii_lowercase();
        Self::ALL.into_iter().find(|role| role.name() == lower)
    }
}

/// What the user chose for each kind of folder. A kind with no entry has not
/// been chosen for: it takes the setting `[list] default_sort`. An entry of
/// `None` is the folder's own order, chosen on purpose.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SortPrefs {
    chosen: BTreeMap<RoleKey, Option<Sort>>,
}

impl SortPrefs {
    /// The choice for `role`, if one was made.
    pub fn get(&self, role: RoleKey) -> Option<Option<Sort>> {
        self.chosen.get(&role).copied()
    }

    pub fn set(&mut self, role: RoleKey, sort: Option<Sort>) {
        self.chosen.insert(role, sort);
    }

    /// The choices as the state file writes them, by key.
    pub fn to_texts(&self) -> BTreeMap<String, String> {
        self.chosen.iter().map(|(role, sort)| (role.name().to_string(), sort_text(*sort))).collect()
    }

    /// The choices a state file holds. An entry that is not understood is
    /// left out: the worst result of a damaged file is a folder in its own order.
    pub fn from_texts(texts: &BTreeMap<String, String>) -> Self {
        let chosen = texts
            .iter()
            .filter_map(|(role, text)| Some((RoleKey::from_name(role)?, parse_sort(text)?)))
            .collect();
        Self { chosen }
    }
}

/// `size descending`, or `none` for the folder's own order.
pub fn sort_text(sort: Option<Sort>) -> String {
    match sort {
        None => "none".to_string(),
        Some(sort) => format!("{} {}", sort.column.name(), if sort.descending { "descending" } else { "ascending" }),
    }
}

fn parse_sort(text: &str) -> Option<Option<Sort>> {
    let lower = text.trim().to_ascii_lowercase();
    if lower == "none" {
        return Some(None);
    }
    let (column, way) = lower.split_once(char::is_whitespace).map_or((lower.as_str(), ""), |(column, way)| (column, way.trim()));
    let column = SortColumn::from_name(column)?;
    let descending = match way {
        "" => column.natural_descending(),
        "descending" | "desc" => true,
        "ascending" | "asc" => false,
        _ => return None,
    };
    Some(Some(Sort { column, descending }))
}

#[cfg(test)]
include!("sort_prefs.tests.rs");
