//! What the footer says about the link the pointer is on: where it goes. A web
//! address is opened and a mail address is written to; the words are made from
//! the link alone, when the footer is drawn, and kept nowhere.

/// The words the footer shows for a link: a verb and the target after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkHint {
    /// `Open ` or `Write to `, with its blank.
    pub verb: &'static str,
    /// The address, or for a mail link the address written to.
    pub target: String,
}

/// The start of a mail link, which the hint leaves off.
const MAILTO: &str = "mailto:";

/// Whether an address in a message is a web address, which the browser opens;
/// anything else that is marked as an address is a mail address.
pub fn is_web_address(address: &str) -> bool {
    let start = address.get(..8).unwrap_or(address).to_ascii_lowercase();
    start.starts_with("http://") || start.starts_with("https://")
}

/// Whether `ch` could make the target read as something it is not: a control
/// code, a zero-width character or a text-direction override.
fn is_hidden(ch: char) -> bool {
    ch.is_control()
        || matches!(ch, '\u{00AD}' | '\u{180E}' | '\u{200B}'..='\u{200F}' | '\u{2028}'..='\u{202E}' | '\u{2060}'..='\u{2069}' | '\u{FEFF}')
}

/// The address a mail link writes to: without `mailto:` (in any case) and
/// without the `?subject=...` that follows it.
fn mail_target(address: &str) -> &str {
    let head = address.get(..MAILTO.len()).filter(|head| head.eq_ignore_ascii_case(MAILTO));
    let rest = if head.is_some() { &address[MAILTO.len()..] } else { address };
    rest.split_once('?').map_or(rest, |(to, _)| to)
}

impl LinkHint {
    /// The hint for the link `address`, as the reader holds it.
    pub fn for_address(address: &str) -> Self {
        let (verb, target) = if is_web_address(address) { ("Open ", address) } else { ("Write to ", mail_target(address)) };
        Self { verb, target: target.chars().filter(|ch| !is_hidden(*ch)).collect() }
    }
}

#[cfg(test)]
include!("link_hint.tests.rs");
