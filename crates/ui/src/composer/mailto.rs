//! A `mailto:` link as a message to start: the recipients in the address and
//! in its `to`, `cc` and `bcc` fields, a subject and a body, read the way RFC
//! 6068 says (percent-decoding, with `+` left as it is). A link is only read,
//! never trusted: a line break or any other control code in an address or the
//! subject would let a link add headers of its own, so such a link is refused
//! whole; the body keeps its line breaks and loses every other control code;
//! every address must look like an address. A bare address in a message is a
//! link to itself. Fields this program does not use (`in-reply-to`,
//! `references`...) are ignored.

use model::Address;

/// The longest link that is read.
const MAX_LINK_BYTES: usize = 16 * 1024;
const MAX_SUBJECT_CHARS: usize = 998;
const MAX_BODY_BYTES: usize = 64 * 1024;
const MAX_RECIPIENTS: usize = 100;

/// What a `mailto:` link asks for.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MailtoDraft {
    pub to: Vec<Address>,
    pub cc: Vec<Address>,
    pub bcc: Vec<Address>,
    pub subject: String,
    pub body: String,
}

/// Why a link was not opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MailtoError {
    /// Not a `mailto:` link or an address.
    NotMail,
    /// A line break or control code in a part that is a header.
    HeaderBreak(&'static str),
    /// An address that does not look like one.
    BadAddress(String),
    /// A part that is not valid UTF-8 once decoded.
    BadEncoding(&'static str),
    TooLong,
}

impl MailtoError {
    /// What the footer says.
    pub fn words(&self) -> String {
        match self {
            Self::NotMail => "that is not a mail link".to_owned(),
            Self::HeaderBreak(field) => format!("this mail link was not opened: its {field} holds a line break or control code"),
            Self::BadAddress(address) => format!("this mail link was not opened: {address} is not an address"),
            Self::BadEncoding(field) => format!("this mail link was not opened: its {field} is not valid text"),
            Self::TooLong => "this mail link was not opened: it is too long".to_owned(),
        }
    }
}

/// The message a link asks for. `text` is `mailto:` and what follows, or a
/// bare address.
pub fn parse_mailto(text: &str) -> Result<MailtoDraft, MailtoError> {
    let text = text.trim();
    if text.len() > MAX_LINK_BYTES {
        return Err(MailtoError::TooLong);
    }
    let rest = match text.get(..7).filter(|start| start.eq_ignore_ascii_case("mailto:")) {
        Some(_) => &text[7..],
        None if looks_like_address(text) => text,
        None => return Err(MailtoError::NotMail),
    };
    let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
    let mut draft = MailtoDraft::default();
    add_recipients(&mut draft.to, path, "address")?;
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        match name.to_ascii_lowercase().as_str() {
            "to" => add_recipients(&mut draft.to, value, "to")?,
            "cc" => add_recipients(&mut draft.cc, value, "cc")?,
            "bcc" => add_recipients(&mut draft.bcc, value, "bcc")?,
            "subject" => draft.subject = header_text(value, "subject")?,
            "body" => draft.body = body_text(value)?,
            _ => {}
        }
    }
    if draft.subject.chars().count() > MAX_SUBJECT_CHARS || draft.body.len() > MAX_BODY_BYTES {
        return Err(MailtoError::TooLong);
    }
    Ok(draft)
}

/// Whether `text` is, all of it, a plain address.
pub fn looks_like_address(text: &str) -> bool {
    let Some((local, domain)) = text.split_once('@') else {
        return false;
    };
    let local_ok = !local.is_empty()
        && local.chars().all(|c| c.is_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-' | '\'' | '!' | '#' | '$' | '&' | '*' | '/' | '=' | '?' | '^' | '`' | '{' | '|' | '}' | '~'));
    let labels: Vec<&str> = domain.split('.').collect();
    let domain_ok = !domain.is_empty()
        && labels.iter().all(|label| !label.is_empty() && !label.starts_with('-') && !label.ends_with('-'))
        && domain.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-');
    local_ok && domain_ok
}

/// Adds the addresses of a comma-separated, percent-encoded list.
fn add_recipients(into: &mut Vec<Address>, encoded: &str, field: &'static str) -> Result<(), MailtoError> {
    let decoded = header_text(encoded, field)?;
    for address in decoded.split(',').map(str::trim).filter(|address| !address.is_empty()) {
        if !looks_like_address(address) {
            return Err(MailtoError::BadAddress(address.chars().take(60).collect()));
        }
        into.push(Address::bare(address));
    }
    if into.len() > MAX_RECIPIENTS {
        return Err(MailtoError::TooLong);
    }
    Ok(())
}

/// The percent-decoded value of a part that ends up in a header: no control
/// code of any kind may be in it, so it cannot carry a header of its own.
fn header_text(encoded: &str, field: &'static str) -> Result<String, MailtoError> {
    let text = decode(encoded, field)?;
    if text.chars().any(|c| c.is_control()) {
        return Err(MailtoError::HeaderBreak(field));
    }
    Ok(text)
}

/// The percent-decoded body: line breaks are kept (one kind of them), the
/// other control codes are dropped.
fn body_text(encoded: &str) -> Result<String, MailtoError> {
    let text = decode(encoded, "body")?.replace("\r\n", "\n").replace('\r', "\n");
    Ok(text.chars().filter(|c| *c == '\n' || *c == '\t' || !c.is_control()).collect())
}

fn decode(encoded: &str, field: &'static str) -> Result<String, MailtoError> {
    let bytes = encoded.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        let pair = if bytes[at] == b'%' { bytes.get(at + 1..at + 3).and_then(hex_pair) } else { None };
        match pair {
            Some(byte) => {
                out.push(byte);
                at += 3;
            }
            None => {
                out.push(bytes[at]);
                at += 1;
            }
        }
    }
    String::from_utf8(out).map_err(|_| MailtoError::BadEncoding(field))
}

fn hex_pair(pair: &[u8]) -> Option<u8> {
    let digit = |byte: u8| char::from(byte).to_digit(16);
    Some(u8::try_from(digit(pair[0])? * 16 + digit(pair[1])?).ok()?)
}

#[cfg(test)]
include!("mailto.tests.rs");
