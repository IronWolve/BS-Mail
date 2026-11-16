//! The sort of the message list: which column, and which way. The list sorts
//! the rows it has loaded, never the whole folder, and says so in its title.
//!
//! "No sort" is the folder's own order, which is the order the mail arrived
//! in, newest first; it is what the list shows until a sort is chosen. The
//! Date sort is by the date the Date column shows, which can differ from the
//! arrival order for mail that was delivered later than it was written.

use model::{Envelope, FolderRole};

use super::subject_key::base_subject;

/// A column the list can be sorted by. Both the Flag and the Star column sort
/// by `Flag`: the messages that carry a mark come first or last.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SortColumn {
    Date,
    /// The sender, or the recipient in Sent and Drafts.
    From,
    Subject,
    Size,
    Flag,
}

impl SortColumn {
    pub const ALL: [SortColumn; 5] = [Self::Date, Self::From, Self::Subject, Self::Size, Self::Flag];

    /// The name the settings and the state file use.
    pub fn name(self) -> &'static str {
        match self {
            Self::Date => "date",
            Self::From => "from",
            Self::Subject => "subject",
            Self::Size => "size",
            Self::Flag => "flag",
        }
    }

    /// The column called `name` (`to` is the sender column in Sent).
    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.trim().to_ascii_lowercase();
        Self::ALL.into_iter().find(|column| column.name() == lower).or((lower == "to").then_some(Self::From))
    }

    /// What the title calls the column: `to` for the sender column over mail
    /// that names its recipient.
    pub fn title_name(self, recipients: bool) -> &'static str {
        match (self, recipients) {
            (Self::From, true) => "to",
            _ => self.name(),
        }
    }

    /// The way a column goes when it is first chosen: the newest, the biggest
    /// and the marked first; names from A.
    pub fn natural_descending(self) -> bool {
        matches!(self, Self::Date | Self::Size | Self::Flag)
    }
}

/// A column and a direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sort {
    pub column: SortColumn,
    pub descending: bool,
}

impl Sort {
    /// The column in the way it goes when first chosen.
    pub fn natural(column: SortColumn) -> Self {
        Self { column, descending: column.natural_descending() }
    }

    /// The setting `[list] default_sort`: `date`, newest first, is the
    /// folder's own order (`Some(None)`); another known column is a sort; an
    /// unknown column is `None`.
    pub fn from_setting(column: &str, descending: bool) -> Option<Option<Self>> {
        let column = SortColumn::from_name(column)?;
        Some((column != SortColumn::Date || !descending).then_some(Self { column, descending }))
    }

    /// The same sort the other way round.
    #[must_use]
    pub fn reversed(self) -> Self {
        Self { descending: !self.descending, ..self }
    }

    /// How the direction is said: the words for what comes first.
    pub fn first_word(self) -> &'static str {
        match (self.column, self.descending) {
            (SortColumn::Date, true) => "newest first",
            (SortColumn::Date, false) => "oldest first",
            (SortColumn::Size, true) => "largest first",
            (SortColumn::Size, false) => "smallest first",
            (SortColumn::Flag, true) => "marked first",
            (SortColumn::Flag, false) => "marked last",
            (_, false) => "A to Z",
            (_, true) => "Z to A",
        }
    }
}

/// The sort after `current` when the key for the next column is pressed: the
/// folder's own order, then From, Subject, Size, Flag, Date and round again.
pub fn next_sort(current: Option<Sort>) -> Option<Sort> {
    use SortColumn::{Date, Flag, From, Size, Subject};
    let next = match current.map(|sort| sort.column) {
        None => From,
        Some(From) => Subject,
        Some(Subject) => Size,
        Some(Size) => Flag,
        Some(Flag) => Date,
        Some(Date) => return None,
    };
    Some(Sort::natural(next))
}

/// The sort the other way round. The folder's own order, newest first, goes
/// to the oldest first.
pub fn reverse_sort(current: Option<Sort>) -> Option<Sort> {
    Some(current.map_or(Sort { column: SortColumn::Date, descending: false }, Sort::reversed))
}

/// The sort after a click on the header of `column`: the same column again
/// reverses it, another one starts in its natural way.
pub fn click_sort(current: Option<Sort>, column: SortColumn) -> Option<Sort> {
    let shown = current.map_or(SortColumn::Date, |sort| sort.column);
    if shown == column { reverse_sort(current) } else { Some(Sort::natural(column)) }
}

/// What a row is ordered by, for one column.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SortKey {
    Number(i128),
    Text(String),
}

/// The value of `envelope` in `column`. A missing value is the lowest: an
/// empty text, or zero. `recipients` is whether the sender column names the
/// recipient for this message (the mail is in Sent or Drafts).
pub fn key_of(column: SortColumn, envelope: &Envelope, recipients: bool) -> SortKey {
    match column {
        SortColumn::Date => SortKey::Number(i128::from(envelope.date.unix)),
        SortColumn::Size => SortKey::Number(i128::from(envelope.size)),
        SortColumn::Flag => SortKey::Number(i128::from(flag_rank(envelope))),
        SortColumn::Subject => SortKey::Text(base_subject(&envelope.subject)),
        SortColumn::From => SortKey::Text(party_text(envelope, recipients)),
    }
}

/// How much a message's marks count: the alert, then the star, then unread.
pub fn flag_rank(envelope: &Envelope) -> u8 {
    let flags = envelope.flags;
    u8::from(flags.alert) * 4 + u8::from(flags.flagged) * 2 + u8::from(!flags.seen)
}

/// The name of the sender (or the first recipient), else the address, in
/// lower case so capitals do not split a list.
fn party_text(envelope: &Envelope, recipients: bool) -> String {
    let people = if recipients {
        if envelope.to.is_empty() { &envelope.cc } else { &envelope.to }
    } else {
        &envelope.from
    };
    let Some(first) = people.first() else {
        return String::new();
    };
    let name = first.name.as_deref().map(str::trim).filter(|name| !name.is_empty());
    name.unwrap_or_else(|| first.email.trim()).to_lowercase()
}

/// Whether mail filed in a folder of `role` is sorted by its recipient.
pub fn sorts_by_recipient(role: FolderRole) -> bool {
    matches!(role, FolderRole::Sent | FolderRole::Drafts)
}

#[cfg(test)]
include!("sort.tests.rs");
