//! Plain-text matching for the mock's Search command. Terms are separated by
//! spaces and all must match, ignoring case. `from:`, `to:`, `subject:` and
//! `tag:` limit a term to one field; any other term is looked for in the
//! addresses, the subject and the tags. No quoting, no other operators.

use model::{Address, Envelope};

/// Whether `envelope` satisfies every term of `query`; an empty query
/// matches everything.
pub(crate) fn matches(envelope: &Envelope, query: &str) -> bool {
    query
        .split_whitespace()
        .all(|term| term_matches(envelope, &term.to_lowercase()))
}

fn term_matches(envelope: &Envelope, term: &str) -> bool {
    match term.split_once(':') {
        Some(("from", value)) => addresses_contain(&envelope.from, value),
        Some(("to", value)) => addresses_contain(&envelope.to, value),
        Some(("subject", value)) => contains(&envelope.subject, value),
        Some(("tag", value)) => envelope.tags.iter().any(|tag| tag.to_lowercase() == value),
        _ => {
            addresses_contain(&envelope.from, term)
                || addresses_contain(&envelope.to, term)
                || addresses_contain(&envelope.cc, term)
                || contains(&envelope.subject, term)
                || envelope.tags.iter().any(|tag| contains(tag, term))
        }
    }
}

fn addresses_contain(addresses: &[Address], needle: &str) -> bool {
    addresses.iter().any(|address| {
        contains(&address.email, needle)
            || address
                .name
                .as_deref()
                .is_some_and(|name| contains(name, needle))
    })
}

fn contains(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(needle)
}

#[cfg(test)]
include!("search.tests.rs");
