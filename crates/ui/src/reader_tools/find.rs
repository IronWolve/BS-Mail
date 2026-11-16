//! The find box over the reading pane: the query being typed, whether the
//! box takes the keys, which match is the current one, and the matches of the
//! body on screen, kept until the query or the body changes.

use std::cell::RefCell;
use std::rc::Rc;

use super::find_text::{MAX_MATCHES, find_matches};
use crate::composer::EditorBuffer;
use crate::widgets::{BodyText, Match};

struct Kept {
    key: u64,
    query: String,
    matches: Rc<Vec<Match>>,
}

pub struct FindState {
    /// The query.
    pub editor: EditorBuffer,
    /// The box takes the typed letters. After Enter it stays on screen with the
    /// result and the reader's keys work again (`n` and `N` go to the next and
    /// the previous match).
    pub editing: bool,
    /// The current match: its place in the list of matches.
    pub current: usize,
    kept: RefCell<Option<Kept>>,
}

impl FindState {
    /// A box with `query` in it, taking the keys.
    pub fn new(query: &str) -> Self {
        Self { editor: EditorBuffer::single_line(query), editing: true, current: 0, kept: RefCell::new(None) }
    }

    pub fn query(&self) -> String {
        self.editor.text()
    }

    /// The matches of the query in `body`, worked out once for each body and query.
    pub fn matches(&self, body: &BodyText) -> Rc<Vec<Match>> {
        let query = self.query();
        let mut kept = self.kept.borrow_mut();
        if let Some(known) = kept.as_ref().filter(|known| known.key == body.key() && known.query == query) {
            return Rc::clone(&known.matches);
        }
        let matches = Rc::new(find_matches(body, &query));
        *kept = Some(Kept { key: body.key(), query, matches: Rc::clone(&matches) });
        matches
    }

    /// What the box says about the result: `3 of 17`, `no match`, or what to type.
    pub fn status(&self, matches: &[Match]) -> String {
        if self.query().is_empty() {
            return "type to find".to_owned();
        }
        match matches.len() {
            0 => "no match".to_owned(),
            n if n >= MAX_MATCHES => format!("{} of {}+", self.current.min(n - 1) + 1, n),
            n => format!("{} of {}", self.current.min(n - 1) + 1, n),
        }
    }
}

#[cfg(test)]
include!("find.tests.rs");
