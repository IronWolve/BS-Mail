//! The wrapped lines of the body being read, kept from one frame to the next.
//!
//! Highlighting and wrapping a body touches all of it, and scrolling asks for
//! that twice per step (how far it can scroll, then what to draw). For mail
//! near the size limit that is a visible lag, so the last result is kept and
//! given again while what it was made from is unchanged. What it was made from
//! is recognised by a fingerprint of the text, the width, and the settings that
//! change the result, so a different message, a resized pane or another setting
//! can never be served the old lines.

use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use model::StyledLine;

use super::wrap::Piece;
use crate::widgets::body_highlight::HighlightRules;

/// The wrapped lines of one body.
pub(super) type Wrapped = Rc<Vec<Vec<Piece>>>;

thread_local! {
    static LAST: RefCell<Option<(u64, Wrapped)>> = const { RefCell::new(None) };
}

/// A number that differs when anything the wrapped lines depend on does.
pub(super) fn fingerprint(
    lines: &[StyledLine],
    width: u16,
    highlight: bool,
    rules: &HighlightRules,
    app_name: &str,
) -> u64 {
    let mut hasher = DefaultHasher::new();
    (width, highlight, format!("{rules:?}"), app_name, lines.len()).hash(&mut hasher);
    for line in lines {
        line.spans.len().hash(&mut hasher);
        for span in &line.spans {
            std::mem::discriminant(&span.role).hash(&mut hasher);
            span.text.hash(&mut hasher);
        }
    }
    hasher.finish()
}

/// The lines last made for this fingerprint, if that was the last body.
pub(super) fn remembered(key: u64) -> Option<Wrapped> {
    LAST.with(|last| last.borrow().as_ref().filter(|(known, _)| *known == key).map(|(_, wrapped)| Rc::clone(wrapped)))
}

/// Keeps `wrapped` as the last body's lines, replacing the one before.
pub(super) fn remember(key: u64, wrapped: &Wrapped) {
    LAST.with(|last| *last.borrow_mut() = Some((key, Rc::clone(wrapped))));
}

#[cfg(test)]
include!("cache.tests.rs");
