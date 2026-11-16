//! The `signature` key of an account file: free text, with `\n` for a line
//! break, that the composer puts into new mail from the account.
//!
//! What the file says is made safe to draw and send: line ends become `\n`,
//! a literal backslash and `n` (typed in a TOML literal string) is a line
//! break too, tabs become four spaces, every other control code is dropped,
//! the blank lines and the blanks at the ends are trimmed, and the text is cut
//! to `MAX_SIGNATURE_CHARS` characters, which no signature needs more than.
//! The cut is not an error: a signature is never a reason for an account not
//! to start.

/// The most characters of a signature that are kept.
pub const MAX_SIGNATURE_CHARS: usize = 2000;

/// The signature the file asks for, cleaned; empty when it has none.
pub fn clean_signature(raw: Option<&str>) -> String {
    let Some(raw) = raw else {
        return String::new();
    };
    let text = raw.replace("\r\n", "\n").replace('\r', "\n").replace("\\n", "\n");
    let kept: String = text
        .chars()
        .filter_map(|ch| match ch {
            '\t' => Some("    ".to_owned()),
            '\n' => Some("\n".to_owned()),
            ch if ch.is_control() => None,
            ch => Some(ch.to_string()),
        })
        .collect();
    let lines: Vec<&str> = kept.lines().map(str::trim_end).collect();
    let first = lines.iter().position(|line| !line.is_empty());
    let last = lines.iter().rposition(|line| !line.is_empty());
    let (Some(first), Some(last)) = (first, last) else {
        return String::new();
    };
    let joined = lines[first..=last].join("\n");
    joined.chars().take(MAX_SIGNATURE_CHARS).collect::<String>().trim_end().to_owned()
}

#[cfg(test)]
include!("account_signature.tests.rs");
