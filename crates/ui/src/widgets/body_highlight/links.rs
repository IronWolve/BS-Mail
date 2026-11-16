//! Addresses in a message: web pages, `mailto:` links and plain email
//! addresses, and `(path/like-tokens)` in parentheses.

use model::BodyRole;

use super::found::Found;

/// Link schemes the highlighter knows.
const SCHEMES: [&str; 2] = ["https://", "http://"];
/// Marks that end a sentence, not a link.
const TRAILING: [char; 9] = ['.', ',', ';', ':', '!', '?', ')', ']', '}'];
const MAILTO: &str = "mailto:";
/// The most characters before and after the `@` of a plain address that are read.
const MAX_LOCAL: usize = 64;
const MAX_DOMAIN: usize = 253;

/// Marks every `http://` and `https://` address, every `mailto:` link and
/// every plain email address up to the next blank, without the punctuation a
/// sentence puts after it. Where two would overlap the first found keeps its
/// text, so the address inside a web link is part of the web link.
pub fn find_links(text: &str, found: &mut Found) {
    find_web_links(text, found);
    find_mailto_links(text, found);
    find_email_addresses(text, found);
}

fn find_web_links(text: &str, found: &mut Found) {
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        let Some(scheme) = SCHEMES.iter().filter_map(|s| rest.find(s).map(|i| (i, s.len()))).min()
        else {
            return;
        };
        let start = at + scheme.0;
        let tail = &text[start..];
        let stop = tail.find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"')).unwrap_or(tail.len());
        let link = tail[..stop].trim_end_matches(TRAILING);
        if link.len() > scheme.1 {
            found.add(start..start + link.len(), BodyRole::Link);
        }
        at = start + stop.max(scheme.1);
    }
}

/// `mailto:` and what follows it, such as `mailto:a@b.org?subject=Hi`.
fn find_mailto_links(text: &str, found: &mut Found) {
    // ASCII lower case keeps every byte position, so the two texts line up.
    let lower = text.to_ascii_lowercase();
    let mut at = 0;
    while let Some(index) = lower[at..].find(MAILTO) {
        let start = at + index;
        let tail = &text[start..];
        let stop = tail.find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"')).unwrap_or(tail.len());
        let link = tail[..stop].trim_end_matches(TRAILING);
        let after_a_letter = text[..start].chars().next_back().is_some_and(char::is_alphanumeric);
        if link.len() > MAILTO.len() && !after_a_letter {
            found.add(start..start + link.len(), BodyRole::Link);
        }
        at = start + stop.max(MAILTO.len());
    }
}

fn is_local_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-')
}

fn is_domain_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '.' | '-')
}

/// Whether `domain` is a host name with at least one dot, no empty part and a
/// last part of letters only (`example.org`, not `v1.2` or `a..b`).
fn is_host_name(domain: &str) -> bool {
    let labels: Vec<&str> = domain.split('.').collect();
    labels.len() >= 2
        && domain.len() <= MAX_DOMAIN
        && labels.iter().all(|label| !label.is_empty() && !label.starts_with('-') && !label.ends_with('-'))
        && labels.last().is_some_and(|last| last.chars().count() >= 2 && last.chars().all(char::is_alphabetic))
}

/// Plain addresses such as `sam.lee@example.org`, as a sentence may hold
/// them: the full stop after one is not part of it.
fn find_email_addresses(text: &str, found: &mut Found) {
    for (at, _) in text.match_indices('@') {
        let local_start = text[..at]
            .char_indices()
            .rev()
            .take_while(|(_, c)| is_local_char(*c))
            .last()
            .map_or(at, |(index, _)| index);
        let local = text[local_start..at].trim_start_matches(['.', '-']);
        let local_start = at - local.len();
        let after = &text[at + 1..];
        let domain = after[..after.find(|c: char| !is_domain_char(c)).unwrap_or(after.len())].trim_end_matches(['.', '-']);
        let inside_a_path = text[..local_start].ends_with('/');
        if !local.is_empty() && local.len() <= MAX_LOCAL && is_host_name(domain) && !inside_a_path {
            found.add(local_start..at + 1 + domain.len(), BodyRole::Link);
        }
    }
}

fn is_path_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '.' | '_' | '/' | '-' | '~')
}

/// Marks a path-like token (one that holds a `/`) that is the whole content
/// of a pair of parentheses; the parentheses stay plain.
pub fn find_paren_paths(text: &str, found: &mut Found) {
    for (open, _) in text.match_indices('(') {
        let inner = &text[open + 1..];
        let length = inner.find(|c: char| !is_path_char(c)).unwrap_or(inner.len());
        let closed = inner[length..].starts_with(')');
        let token = &inner[..length];
        if closed && token.contains('/') && !token.starts_with("//") {
            found.add(open + 1..open + 1 + length, BodyRole::Path);
        }
    }
}

#[cfg(test)]
include!("links.tests.rs");
