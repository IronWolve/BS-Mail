//! The matches the rules found in one piece of text: byte ranges with a
//! role. A range that overlaps one already found is refused, so the rule
//! that runs first wins.

use std::ops::Range;

use model::{BodyRole, StyledSpan};

#[derive(Debug, Default)]
pub struct Found {
    items: Vec<(Range<usize>, BodyRole)>,
}

impl Found {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records `range` as `role` unless it is empty or overlaps an earlier
    /// match. Returns whether it was recorded.
    pub fn add(&mut self, range: Range<usize>, role: BodyRole) -> bool {
        let clear = range.start < range.end
            && self
                .items
                .iter()
                .all(|(other, _)| range.end <= other.start || range.start >= other.end);
        if clear {
            self.items.push((range, role));
        }
        clear
    }

    /// The text as spans in order: the matches with their roles and plain
    /// text between them.
    pub fn into_spans(mut self, text: &str) -> Vec<StyledSpan> {
        self.items.sort_by_key(|(range, _)| range.start);
        let mut spans = Vec::new();
        let mut at = 0;
        for (range, role) in self.items {
            if range.start > at {
                spans.push(span(&text[at..range.start], BodyRole::Text));
            }
            spans.push(span(&text[range.start..range.end], role));
            at = range.end;
        }
        if at < text.len() {
            spans.push(span(&text[at..], BodyRole::Text));
        }
        spans
    }
}

fn span(text: &str, role: BodyRole) -> StyledSpan {
    StyledSpan {
        text: text.to_string(),
        role,
    }
}

#[cfg(test)]
include!("found.tests.rs");
