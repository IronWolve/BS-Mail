//! Looking at what the person typed before any of it goes near a header or
//! a command line. Anything with a line break or another control character
//! in an address, a name or a subject is refused, never cleaned up: a
//! changed message is a message nobody wrote.

use std::fmt;

use model::Address;

use super::limits::{MAX_ADDRESS_BYTES, MAX_NAME_CHARS, MAX_SUBJECT_CHARS};

/// Why a message was not sent, as one plain sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal(String);

impl Refusal {
    pub fn new(reason: impl Into<String>) -> Self {
        Self(reason.into())
    }

    pub fn reason(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An address and name that passed every check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mailbox {
    pub name: Option<String>,
    pub address: String,
}

/// Characters that never belong in a header or a command: controls, and the
/// Unicode line and paragraph separators some programs treat as line breaks.
pub fn is_forbidden(c: char) -> bool {
    c.is_control() || c == '\u{2028}' || c == '\u{2029}'
}

/// A piece of text for a message, shown back with anything odd replaced and
/// cut short, so a refusal never echoes a line break.
pub fn shown(text: &str) -> String {
    let mut out: String = text
        .chars()
        .take(60)
        .map(|c| if is_forbidden(c) { '?' } else { c })
        .collect();
    if text.chars().count() > 60 {
        out.push_str("...");
    }
    out
}

/// Refuses `text` when it holds a forbidden character. `what` names the part
/// ("the subject", "the name of a recipient").
pub fn check_clean(what: &str, text: &str) -> Result<(), Refusal> {
    if text.chars().any(is_forbidden) {
        return Err(Refusal::new(format!(
            "{what} contains a line break or another control character; nothing was sent"
        )));
    }
    Ok(())
}

/// The subject, checked.
pub fn check_subject(subject: &str) -> Result<(), Refusal> {
    check_clean("the subject", subject)?;
    if subject.chars().count() > MAX_SUBJECT_CHARS {
        return Err(Refusal::new(format!(
            "the subject is too long (over {MAX_SUBJECT_CHARS} characters); nothing was sent"
        )));
    }
    Ok(())
}

/// A name and address, checked. `role` says whose it is ("the sender",
/// "a recipient") for the reason text.
pub fn check_mailbox(role: &str, who: &Address) -> Result<Mailbox, Refusal> {
    check_clean(&format!("the address of {role}"), &who.email)?;
    if let Some(problem) = address_problem(&who.email) {
        return Err(Refusal::new(format!(
            "\"{}\", the address of {role}, {problem}; nothing was sent",
            shown(&who.email)
        )));
    }
    if let Some(raw) = who.name.as_deref() {
        // Before any trimming: a line break at the end is still a line break.
        check_clean(&format!("the name of {role}"), raw)?;
    }
    let name = match who.name.as_deref().map(|name| name.trim_matches(' ')) {
        None | Some("") => None,
        Some(name) => {
            if name.chars().count() > MAX_NAME_CHARS {
                return Err(Refusal::new(format!(
                    "the name of {role} is too long (over {MAX_NAME_CHARS} characters); \
                     nothing was sent"
                )));
            }
            Some(name.to_owned()).filter(|name| *name != who.email)
        }
    };
    Ok(Mailbox { name, address: who.email.clone() })
}

/// What is wrong with a bare address, or `None` when it can be used. Plain
/// ASCII `local@domain` only: no quoted parts, comments or address literals.
pub fn address_problem(address: &str) -> Option<&'static str> {
    if address.len() > MAX_ADDRESS_BYTES {
        return Some("is too long");
    }
    if !address.is_ascii() {
        return Some("has characters outside plain ASCII, which this program cannot send yet");
    }
    let Some((local, domain)) = address.split_once('@') else {
        return Some("is not a mail address (it has no @)");
    };
    if domain.contains('@') {
        return Some("has more than one @");
    }
    if !local_part_ok(local) {
        return Some("has a part before the @ that is not allowed");
    }
    if !domain_ok(domain) {
        return Some("has a part after the @ that is not a host name");
    }
    None
}

fn local_part_ok(local: &str) -> bool {
    const SPECIAL: &str = "!#$%&'*+-/=?^_`{|}~";
    let ok_char = |c: char| c.is_ascii_alphanumeric() || SPECIAL.contains(c);
    !local.is_empty()
        && local.len() <= 64
        && local.split('.').all(|piece| !piece.is_empty() && piece.chars().all(ok_char))
}

fn domain_ok(domain: &str) -> bool {
    let label_ok = |label: &str| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    };
    !domain.is_empty() && domain.len() <= 253 && domain.split('.').all(label_ok)
}

#[cfg(test)]
include!("check.tests.rs");
