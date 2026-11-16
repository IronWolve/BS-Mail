//! The signature in a message body: the `-- ` line and the account's text,
//! put under the cursor of a new message and above the quoted text of a reply
//! or a forward, once, and swapped for another account's when the From line
//! changes.

/// The line that starts a signature, with the blank every mail program
/// expects after the dashes.
pub const SEPARATOR: &str = "-- ";

/// The signature as it stands in a body: the separator line, then the text.
pub fn block(signature: &str) -> String {
    format!("{SEPARATOR}\n{signature}")
}

/// `base` with the signature in it. A body that is empty gets it two lines
/// down, so the cursor at the start has a line of its own above it; a reply or
/// a forward starts with those two blank lines, and the signature goes into
/// them, above the quoted text. A body that already holds the signature, or
/// that is text of its own with no room made for one (a draft being
/// continued), is left as it is.
pub fn insert(base: &str, signature: &str) -> String {
    if signature.is_empty() || find_block(base, signature).is_some() {
        return base.to_string();
    }
    if base.trim().is_empty() {
        return format!("\n\n{}", block(signature));
    }
    match base.strip_prefix("\n\n") {
        Some(rest) => format!("\n\n{}\n\n{rest}", block(signature)),
        None => base.to_string(),
    }
}

/// `text` with the signature `old` replaced by `new`, or taken out when `new`
/// is empty; `None` when `text` does not hold `old` as a whole block, which
/// is to say the user changed or removed it.
pub fn swap(text: &str, old: &str, new: &str) -> Option<String> {
    if old.is_empty() {
        return None;
    }
    let (start, end) = find_block(text, old)?;
    let after = &text[end..];
    let tail = if new.is_empty() { after.strip_prefix("\n\n").unwrap_or(after) } else { after };
    let middle = if new.is_empty() { String::new() } else { block(new) };
    Some(format!("{}{middle}{tail}", &text[..start]))
}

/// Where the block of `signature` stands in `text`: from the start of a line
/// to the end of one, so a longer text that begins with the same words is not
/// taken for it.
fn find_block(text: &str, signature: &str) -> Option<(usize, usize)> {
    let block = block(signature);
    let mut from = 0;
    while let Some(found) = text[from..].find(&block) {
        let start = from + found;
        let end = start + block.len();
        let line_start = start == 0 || text[..start].ends_with('\n');
        let line_end = end == text.len() || text[end..].starts_with('\n');
        if line_start && line_end {
            return Some((start, end));
        }
        from = start + SEPARATOR.len();
    }
    None
}

#[cfg(test)]
include!("signature.tests.rs");
