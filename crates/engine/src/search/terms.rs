//! Turning `field:value` into a term, and checking the text a term carries.

use super::ast::{Status, Term};
use super::dates::parse_date;
use super::error::SearchError;
use super::sizes::{Bound, parse_size};

/// The longest text one term may carry, in bytes.
pub const MAX_TERM_BYTES: usize = 1000;

/// The fields a query may use, for the sentence that lists them.
const FIELDS: &str = "from, to, cc, subject, body, tag, is, before, after, size and folder";

/// Text to look for, checked: not empty, not too long, and free of the one
/// character no mail protocol can carry.
pub fn check_text(text: String) -> Result<String, SearchError> {
    if text.is_empty() {
        return Err(SearchError::new("a search word or phrase is empty"));
    }
    if text.len() > MAX_TERM_BYTES {
        return Err(SearchError::new(format!(
            "a search word or phrase is longer than {MAX_TERM_BYTES} bytes"
        )));
    }
    if text.contains('\0') {
        return Err(SearchError::new(
            "a search word contains a character that cannot be sent to a mail server",
        ));
    }
    Ok(text)
}

/// The term for `name:value`.
pub fn field_term(name: &str, value: &str) -> Result<Term, SearchError> {
    if name == "has" && value.eq_ignore_ascii_case("attachment") {
        return Err(SearchError::new("has:attachment is not supported by server-side search yet"));
    }
    if !FIELDS_WITH_VALUES.contains(&name) {
        return Err(SearchError::new(format!(
            "unknown field \"{name}:\"; the fields are {FIELDS} (put the text in quotes to search for it as it is)"
        )));
    }
    if value.is_empty() {
        return Err(SearchError::new(format!(
            "{name}: needs something after the colon, such as {name}:{}",
            example(name)
        )));
    }
    match name {
        "from" => Ok(Term::From(check_text(value.to_owned())?)),
        "to" => Ok(Term::To(check_text(value.to_owned())?)),
        "cc" => Ok(Term::Cc(check_text(value.to_owned())?)),
        "subject" => Ok(Term::Subject(check_text(value.to_owned())?)),
        "body" => Ok(Term::Body(check_text(value.to_owned())?)),
        "folder" => Ok(Term::Folder(check_text(value.to_owned())?)),
        "tag" => keyword(value).map(Term::Tag),
        "is" => status(value).map(Term::Is),
        "before" => parse_date(name, value).map(Term::Before),
        "after" => parse_date(name, value).map(Term::After),
        _ => parse_size(value).map(|bound| match bound {
            Bound::Larger(bytes) => Term::Larger(bytes),
            Bound::Smaller(bytes) => Term::Smaller(bytes),
        }),
    }
}

const FIELDS_WITH_VALUES: [&str; 11] =
    ["from", "to", "cc", "subject", "body", "tag", "is", "before", "after", "size", "folder"];

fn example(name: &str) -> &'static str {
    match name {
        "tag" => "work",
        "is" => "unread",
        "before" | "after" => "2025-02-05",
        "size" => ">1M",
        "folder" => "Projects",
        _ => "alex",
    }
}

fn status(value: &str) -> Result<Status, SearchError> {
    match value.to_ascii_lowercase().as_str() {
        "unread" => Ok(Status::Unread),
        "read" => Ok(Status::Read),
        "starred" | "flagged" => Ok(Status::Flagged),
        "answered" => Ok(Status::Answered),
        "draft" => Ok(Status::Draft),
        _ => Err(SearchError::new(format!(
            "is: knows unread, read, starred, flagged, answered and draft; \"{value}\" is not one of them"
        ))),
    }
}

/// An IMAP keyword is a plain word: letters, digits and a few marks.
fn keyword(value: &str) -> Result<String, SearchError> {
    let plain = value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '$' | ':'));
    if plain && value.len() <= 100 {
        Ok(value.to_owned())
    } else {
        Err(SearchError::new(format!(
            "tag: takes a plain word of letters, digits and - _ . $ : only; \"{value}\" is not one"
        )))
    }
}

#[cfg(test)]
include!("terms.tests.rs");
