//! The rules for names a person types that then go to the server: a folder's
//! name and a tag. Both screens and the engine check with these, so a name
//! the screen lets through is one the engine sends, and one it refuses is
//! refused in the same words wherever it is typed. Every refusal is a plain
//! sentence, ready for the footer.

/// The most characters a folder's name holds.
pub const MAX_FOLDER_NAME: usize = 100;

/// The most characters a tag holds.
pub const MAX_TAG: usize = 50;

/// Keywords that servers and other programs keep for their own bookkeeping.
/// They are never shown as tags, so a tag with one of these names could be set
/// and never seen again; they cannot be typed as a tag either. (The same names
/// are left out of the tags a message shows.)
pub const BOOKKEEPING_KEYWORDS: [&str; 5] = ["nonjunk", "junk", "notjunk", "mdnsent", "phishing"];

/// The characters a name cannot hold in a folder: the two list wildcards, the
/// quote and the backslash. Servers differ in what else they take, and refuse
/// the rest in their own words.
const FOLDER_FORBIDDEN: [char; 4] = ['*', '%', '"', '\\'];

/// The characters an IMAP atom cannot hold besides spaces and control
/// characters (the specials of RFC 3501): a keyword is an atom.
const ATOM_SPECIALS: [char; 8] = ['(', ')', '{', '}', '%', '*', '"', '\\'];

/// A folder's name as it will be written, or why it cannot be one. The
/// engine adds what depends on the server (its separator, the names that
/// exist). The name is trimmed; a name that is only dots is refused.
pub fn check_folder_name(text: &str) -> Result<String, String> {
    let name = text.trim();
    if name.is_empty() {
        return Err("type a name for the folder".to_owned());
    }
    if name.chars().count() > MAX_FOLDER_NAME {
        return Err(format!("a folder name is at most {MAX_FOLDER_NAME} characters"));
    }
    if name.chars().any(char::is_control) {
        return Err("a folder name cannot hold line breaks or other control characters".to_owned());
    }
    if let Some(found) = name.chars().find(|ch| FOLDER_FORBIDDEN.contains(ch)) {
        return Err(format!("a folder name cannot hold the character {found}"));
    }
    if name.chars().all(|ch| ch == '.') {
        return Err("a folder name cannot be only dots".to_owned());
    }
    Ok(name.to_owned())
}

/// A tag as it will be sent, or why it cannot be one. A tag is an IMAP
/// keyword, which is a plain word: printable ASCII with no space and none of
/// the atom specials, and it does not start with `$` or `\`, which the server
/// keeps for its own marks (the alert mark is `$Important`, which has its own
/// key).
pub fn check_tag(text: &str) -> Result<String, String> {
    let tag = text.trim();
    if tag.is_empty() {
        return Err("type a name for the tag".to_owned());
    }
    if tag.chars().count() > MAX_TAG {
        return Err(format!("a tag is at most {MAX_TAG} characters"));
    }
    if tag.chars().any(char::is_whitespace) {
        return Err("a tag cannot hold spaces; use - or _ between words".to_owned());
    }
    if !tag.chars().all(|ch| ch.is_ascii_graphic()) {
        return Err("a tag uses plain letters, digits and marks like - _ . : only (no accents)".to_owned());
    }
    if let Some(found) = tag.chars().find(|ch| ATOM_SPECIALS.contains(ch) || *ch == ']') {
        return Err(format!("a tag cannot hold the character {found}"));
    }
    if tag.starts_with('$') {
        return Err("a tag cannot start with $: the server keeps those for its own marks".to_owned());
    }
    if BOOKKEEPING_KEYWORDS.contains(&tag.to_ascii_lowercase().as_str()) {
        return Err("that name is one the server uses for its own bookkeeping; pick another".to_owned());
    }
    Ok(tag.to_owned())
}

#[cfg(test)]
include!("names.tests.rs");
