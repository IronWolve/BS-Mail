//! Completing the address being typed in To, Cc and Bcc: up to six matches
//! from the address book under the field, chosen with Down and Up, taken with
//! Tab or Enter, dismissed with Escape.
//!
//! The entry being typed is the text between the commas around the cursor. A
//! match list is made again after each edit (`ComposeForm::refresh_suggestions`,
//! given the book by the screen) and the keys here act on the list while it
//! shows.

use model::Address;
use surface::{KeyEvent, LogicalKey, NamedKey};

use super::address_book::AddressBook;
use super::address_list::{emails_in, format};
use super::form::{ComposeForm, Field, Progress};
use super::graphemes::{count, is_blank, list};
use super::input::Outcome;

/// The most matches shown.
pub const MAX_SUGGESTIONS: usize = 6;

/// The matches for what is typed, and which of them is chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestions {
    pub items: Vec<Address>,
    /// The row Tab and Enter take; `None` when the user has not chosen one
    /// and the text looks like a whole address.
    pub cursor: Option<usize>,
}

/// Where the entry the cursor is in lies in a one-line field, in graphemes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Span {
    /// The first grapheme that is not a blank.
    pub start: usize,
    /// Up to the comma that ends the entry, or the end of the text.
    pub end: usize,
    /// How far the typing has got: the cursor, or the start when the cursor
    /// is in the blanks before it.
    pub typed_to: usize,
}

/// The entry of `line` around grapheme `cursor`. Commas and semicolons in
/// quotes or in angle brackets do not end an entry.
pub(super) fn current_entry(line: &str, cursor: usize) -> Span {
    let graphemes = list(line);
    let (mut quoted, mut escaped, mut angle) = (false, false, false);
    let (mut start, mut end) = (0, graphemes.len());
    for (index, grapheme) in graphemes.iter().enumerate() {
        match *grapheme {
            _ if escaped => escaped = false,
            "\\" if quoted => escaped = true,
            "\"" => quoted = !quoted,
            "<" if !quoted => angle = true,
            ">" if !quoted => angle = false,
            "," | ";" if !quoted && !angle => {
                if index < cursor {
                    start = index + 1;
                } else {
                    end = index;
                    break;
                }
            }
            _ => {}
        }
    }
    while start < end && graphemes.get(start).is_some_and(|grapheme| is_blank(grapheme)) {
        start += 1;
    }
    Span { start, end, typed_to: cursor.clamp(start, end) }
}

impl ComposeForm {
    /// Makes the match list again for the text and cursor of the focused
    /// field. None while the field is not a recipient field, the message is
    /// being sent, text is selected, nothing is typed in the entry yet, the
    /// user closed the list for this text, or nothing in the book matches. A
    /// list that comes out the same keeps the row chosen in it.
    pub fn refresh_suggestions(&mut self, book: &AddressBook) {
        let before = self.suggest.take();
        let focus = self.focus;
        if self.progress != Progress::Writing || !matches!(focus, Field::To | Field::Cc | Field::Bcc) {
            return;
        }
        let buffer = self.buffer(focus);
        let text = buffer.text();
        if buffer.selection().is_some() || self.suggest_off == Some((focus, text.clone())) {
            return;
        }
        let graphemes = list(&text);
        let span = current_entry(&text, buffer.cursor().col);
        let typed: String = graphemes[span.start..span.typed_to].concat();
        if typed.is_empty() {
            return;
        }
        let others: String = graphemes[..span.start].concat() + &graphemes[span.end..].concat();
        let items: Vec<Address> = book
            .matches(&typed, MAX_SUGGESTIONS + 1, &emails_in(&others))
            .into_iter()
            .filter(|address| !address.email.eq_ignore_ascii_case(typed.trim()))
            .take(MAX_SUGGESTIONS)
            .collect();
        if items.is_empty() {
            return;
        }
        // A text with an @ in it is an address typed on purpose, so Enter and
        // Tab go on to the next field unless a match was chosen with Down.
        let cursor = if typed.contains('@') { None } else { Some(0) };
        self.suggest = Some(match before {
            Some(shown) if shown.items == items => Suggestions { items, cursor: shown.cursor },
            _ => Suggestions { items, cursor },
        });
    }

    /// Writes the chosen match into the field in place of the entry being
    /// typed, followed by a comma and a blank when it is the last entry.
    /// False when no match is chosen.
    pub fn accept_suggestion(&mut self) -> bool {
        let Some(Suggestions { items, cursor: Some(chosen) }) = self.suggest.take() else {
            return false;
        };
        let Some(address) = items.get(chosen) else {
            return false;
        };
        let mut text = format(std::slice::from_ref(address));
        let buffer = self.focused();
        let line = buffer.lines()[0].clone();
        let span = current_entry(&line, buffer.cursor().col);
        if span.end >= count(&line) {
            text.push_str(", ");
        }
        self.suggest_off = None;
        self.focused().replace_cols(span.start, span.end, &text)
    }

    /// Closes the list until the text of the field changes.
    fn dismiss_suggestions(&mut self) {
        let focus = self.focus;
        self.suggest_off = Some((focus, self.buffer(focus).text()));
        self.suggest = None;
    }
}

/// A key that the match list takes while it shows: Down and Up choose a row,
/// Tab and Enter take the chosen one, Escape closes the list. `None` for any
/// other key, and for Tab and Enter when no row is chosen: they act as they
/// always do.
pub fn suggestion_key(form: &mut ComposeForm, key: &KeyEvent) -> Option<Outcome> {
    let held = key.modifiers;
    let LogicalKey::Named(named) = key.key else {
        return None;
    };
    let shown = form.suggest.as_ref()?;
    if held.alt || held.meta || held.ctrl {
        return None;
    }
    let (rows, chosen) = (shown.items.len(), shown.cursor);
    let mut moved = |cursor: Option<usize>| {
        if let Some(shown) = form.suggest.as_mut() {
            shown.cursor = cursor;
        }
        Some(Outcome::Changed)
    };
    match named {
        NamedKey::Down => moved(match chosen {
            None => Some(0),
            Some(row) if row + 1 < rows => Some(row + 1),
            Some(_) => None,
        }),
        NamedKey::Up => moved(match chosen {
            None => Some(rows.saturating_sub(1)),
            Some(0) => None,
            Some(row) => Some(row - 1),
        }),
        NamedKey::Tab | NamedKey::Enter if chosen.is_some() && !held.shift => {
            form.accept_suggestion().then_some(Outcome::Changed)
        }
        NamedKey::Escape => {
            form.dismiss_suggestions();
            Some(Outcome::Changed)
        }
        _ => None,
    }
}

#[cfg(test)]
include!("suggest.tests.rs");
