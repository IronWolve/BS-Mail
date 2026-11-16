//! The headers that tie a message to the ones it answers: `In-Reply-To` and
//! `References`, read from what a server or a file says and made safe to keep.
//!
//! Both come from outside and can be hostile, so what is kept is bounded in
//! every way: only the first `MAX_HEADER_BYTES` of a header are read, an id
//! that is too long is dropped, control characters are dropped from what is
//! kept, and a message keeps at most `MAX_REFERENCES` ids of its `References`
//! (the first, which names the start of the conversation, and the last ones,
//! which name the messages it answers) and `MAX_IN_REPLY_TO` of the other.

use std::collections::HashSet;

/// The most of a header's text that is read.
pub const MAX_HEADER_BYTES: usize = 64 * 1024;

/// The most ids kept from one `References` header.
pub const MAX_REFERENCES: usize = 50;

/// The most ids kept from one `In-Reply-To` header.
pub const MAX_IN_REPLY_TO: usize = 4;

/// The longest id kept, in characters, without its angle brackets. A longer
/// one is not an id any mail program wrote.
pub const MAX_ID_LEN: usize = 255;

/// What a message says about the messages it follows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ThreadHeaders {
    /// The ids of `In-Reply-To`, cleaned (see `normal_id`).
    pub in_reply_to: Vec<String>,
    /// The ids of `References`, oldest first, cleaned.
    pub references: Vec<String>,
}

impl ThreadHeaders {
    /// Reads the two headers' text: each may be the bare value or the whole
    /// header line with its name, and either may be empty.
    pub fn parse(in_reply_to: &[u8], references: &[u8]) -> Self {
        let mut refs = ids_in(references, MAX_HEADER_BYTES);
        keep_ends(&mut refs, MAX_REFERENCES);
        let mut parents = ids_in(in_reply_to, MAX_HEADER_BYTES);
        parents.truncate(MAX_IN_REPLY_TO);
        Self { in_reply_to: parents, references: refs }
    }

    /// Nothing is said about any other message.
    pub fn is_empty(&self) -> bool {
        self.in_reply_to.is_empty() && self.references.is_empty()
    }
}

/// An id as it is compared: the text between the angle brackets (or the bare
/// text), without controls and spaces, in lower case. `None` when nothing is
/// left or it is longer than `MAX_ID_LEN`.
pub fn normal_id(text: &str) -> Option<String> {
    let inner = text.trim().trim_start_matches('<').trim_end_matches('>');
    let mut out = String::with_capacity(inner.len().min(MAX_ID_LEN));
    for ch in inner.chars() {
        if ch.is_control() || ch.is_whitespace() {
            continue;
        }
        out.extend(ch.to_lowercase());
    }
    (!out.is_empty() && out.chars().count() <= MAX_ID_LEN).then_some(out)
}

/// The ids written between angle brackets in the first `limit` bytes of
/// `raw`, in order and without repeats. Text outside the brackets (the header's
/// name, comments) is ignored; a header with no brackets at all is read as
/// words that look like ids (they hold an `@`).
fn ids_in(raw: &[u8], limit: usize) -> Vec<String> {
    let text = String::from_utf8_lossy(&raw[..raw.len().min(limit)]);
    let mut ids: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut take = |body: &str| {
        if let Some(id) = normal_id(body)
            && seen.insert(id.clone())
        {
            ids.push(id);
        }
    };
    let mut rest: &str = &text;
    while let Some(open) = rest.find('<') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('>') else {
            break;
        };
        // A `<` inside the text before the `>` starts the id again there.
        take(after[..close].rsplit('<').next().unwrap_or(""));
        rest = &after[close + 1..];
    }
    if !text.contains('<') {
        text.split_whitespace().filter(|word| word.contains('@')).for_each(|word| take(word));
    }
    ids
}

/// Keeps the first id and the last `keep - 1`, when there are more than `keep`.
fn keep_ends(ids: &mut Vec<String>, keep: usize) {
    if ids.len() <= keep || keep == 0 {
        return;
    }
    let tail = ids.split_off(ids.len() - (keep - 1));
    ids.truncate(1);
    ids.extend(tail);
}

#[cfg(test)]
include!("thread_headers.tests.rs");
