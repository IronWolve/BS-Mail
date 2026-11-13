//! A signature name after a dash and the application's own name.

use model::BodyRole;

use super::found::Found;

/// Longest signature name, in characters.
const MAX_NAME_CHARS: usize = 40;

/// Marks the name after a leading `—` or `--` and a blank. `text` must start
/// a line. The dash and the blank stay plain.
pub fn find_signature(text: &str, found: &mut Found) {
    let indent = text.len() - text.trim_start().len();
    let rest = &text[indent..];
    let dash = ["\u{2014}", "--"].into_iter().find_map(|dash| rest.strip_prefix(dash).map(|_| dash.len()));
    let Some(dash) = dash else {
        return;
    };
    let after = &rest[dash..];
    let name = after.trim();
    if !after.starts_with(' ') || name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
        return;
    }
    let start = indent + dash + (after.len() - after.trim_start().len());
    found.add(start..start + name.len(), BodyRole::Signature);
}

/// Marks every whole-word occurrence of the application's display name.
pub fn find_app_name(text: &str, name: &str, found: &mut Found) {
    if name.is_empty() {
        return;
    }
    for (start, _) in text.match_indices(name) {
        let end = start + name.len();
        let before = text[..start].chars().next_back();
        let after = text[end..].chars().next();
        let word = |c: char| c.is_alphanumeric() || c == '_';
        if !before.is_some_and(word) && !after.is_some_and(word) {
            found.add(start..end, BodyRole::Name);
        }
    }
}

#[cfg(test)]
include!("names.tests.rs");
