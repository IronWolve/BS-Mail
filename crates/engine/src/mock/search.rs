//! The mock's Search command. It reads the query with the real parser, so a
//! query the real search refuses is refused here with the same sentence, and
//! it answers from the fixture's envelopes: addresses, subject, tags, flags,
//! date and size. A bare word is looked for in the addresses, the subject
//! and the tags, ignoring case. A query the fixture cannot answer (`body:`:
//! it holds no message text) is a failure, never an empty list that would
//! pass for "no mail".

use std::collections::HashSet;

use model::{Address, Envelope, Folder, FolderId};

use super::convert_date::days_from_civil;
use crate::search::{Date, Expr, Status, Term, compile, parse};

const SECONDS_PER_DAY: i64 = 86_400;

/// A query that was read and can be answered from the fixture.
pub(crate) struct Plan {
    expr: Option<Expr>,
    /// The folders `folder:` terms name; `None` when the query names none.
    folders: Option<HashSet<FolderId>>,
}

/// Reads `query`, or gives the sentence that says why it cannot be searched.
pub(crate) fn plan(query: &str, folders: &[Folder]) -> Result<Plan, String> {
    let parsed = parse(query).map_err(|error| error.to_string())?;
    compile(parsed.expr.as_ref()).map_err(|error| error.to_string())?;
    if parsed.expr.as_ref().is_some_and(needs_message_text) {
        return Err("the mock fixture holds no message text, so body: cannot be searched".to_owned());
    }
    let named = if parsed.folders.is_empty() {
        None
    } else {
        let wanted: Vec<String> = parsed.folders.iter().map(|text| text.to_lowercase()).collect();
        let found: HashSet<FolderId> = folders
            .iter()
            .filter(|folder| wanted.iter().all(|text| contains(&folder.path, text) || contains(&folder.name, text)))
            .map(|folder| folder.id)
            .collect();
        if found.is_empty() {
            return Err(format!("no folder matches {}", parsed.folders.join(" and ")));
        }
        Some(found)
    };
    Ok(Plan { expr: parsed.expr, folders: named })
}

impl Plan {
    /// Whether `envelope` is in the searched folders and satisfies the query;
    /// a query with nothing but folder terms matches every message there.
    pub(crate) fn matches(&self, envelope: &Envelope) -> bool {
        if self.folders.as_ref().is_some_and(|set| !set.contains(&envelope.folder)) {
            return false;
        }
        self.expr.as_ref().is_none_or(|expr| eval(expr, envelope))
    }
}

fn needs_message_text(expr: &Expr) -> bool {
    match expr {
        Expr::Term(term) => matches!(term, Term::Body(_)),
        Expr::Not(inner) => needs_message_text(inner),
        Expr::And(items) | Expr::Or(items) => items.iter().any(needs_message_text),
    }
}

fn eval(expr: &Expr, envelope: &Envelope) -> bool {
    match expr {
        Expr::And(items) => items.iter().all(|item| eval(item, envelope)),
        Expr::Or(items) => items.iter().any(|item| eval(item, envelope)),
        Expr::Not(inner) => !eval(inner, envelope),
        Expr::Term(term) => term_matches(term, envelope),
    }
}

fn term_matches(term: &Term, envelope: &Envelope) -> bool {
    match term {
        Term::Any(text) => {
            let text = text.to_lowercase();
            addresses_contain(&envelope.from, &text)
                || addresses_contain(&envelope.to, &text)
                || addresses_contain(&envelope.cc, &text)
                || contains(&envelope.subject, &text)
                || envelope.tags.iter().any(|tag| contains(tag, &text))
        }
        Term::From(text) => addresses_contain(&envelope.from, &text.to_lowercase()),
        Term::To(text) => addresses_contain(&envelope.to, &text.to_lowercase()),
        Term::Cc(text) => addresses_contain(&envelope.cc, &text.to_lowercase()),
        Term::Subject(text) => contains(&envelope.subject, &text.to_lowercase()),
        Term::Tag(word) => envelope.tags.iter().any(|tag| tag.eq_ignore_ascii_case(word)),
        Term::Is(status) => has_status(envelope, *status),
        Term::Before(day) => local_day(envelope) < day_number(*day),
        Term::After(day) => local_day(envelope) >= day_number(*day),
        Term::Larger(bytes) => envelope.size > u64::from(*bytes),
        Term::Smaller(bytes) => envelope.size < u64::from(*bytes),
        // Folders are settled before any message is looked at; the body is
        // refused when the query is read.
        Term::Body(_) | Term::Folder(_) => false,
    }
}

fn has_status(envelope: &Envelope, status: Status) -> bool {
    let flags = &envelope.flags;
    match status {
        Status::Unread => !flags.seen,
        Status::Read => flags.seen,
        Status::Flagged => flags.flagged,
        Status::Answered => flags.answered,
        Status::Draft => flags.draft,
    }
}

/// The day the message is dated, in its own time zone, as days since the epoch.
fn local_day(envelope: &Envelope) -> i64 {
    let local = envelope.date.unix + i64::from(envelope.date.offset_minutes) * 60;
    local.div_euclid(SECONDS_PER_DAY)
}

fn day_number(day: Date) -> i64 {
    days_from_civil(i64::from(day.year), i64::from(day.month), i64::from(day.day))
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

/// `needle` is already in lower case.
fn contains(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(needle)
}

#[cfg(test)]
include!("search.tests.rs");
