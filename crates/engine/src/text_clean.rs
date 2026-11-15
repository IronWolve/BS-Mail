//! Removing characters that must never reach a screen: control codes that a
//! terminal would obey, and, in header text, the characters that change the
//! direction text runs in.

/// Drops control characters other than the line break and tab. Used on text
/// that came from the network before anything draws it.
pub fn strip_controls(text: &str) -> String {
    text.chars()
        .filter(|c| *c == '\n' || *c == '\t' || !c.is_control())
        .collect()
}

/// The characters that make later text run in another direction: embeddings
/// and overrides (U+202A to U+202E) and isolates (U+2066 to U+2069). A sender
/// can use them to make `invoice.exe` read as `invoice.fdp`. The plain
/// direction marks (U+200E, U+200F) are left alone: right-to-left names need
/// them.
pub fn is_direction_override(c: char) -> bool {
    matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

/// Characters that take no room on screen: format characters such as the
/// zero-width space and joiners, direction marks and overrides, the byte order
/// mark, variation selectors and tag characters. Drawing skips them.
fn is_format(c: char) -> bool {
    matches!(
        c,
        '\u{AD}'
            | '\u{61C}'
            | '\u{180E}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{206F}'
            | '\u{FE00}'..='\u{FE0F}'
            | '\u{FEFF}'
            | '\u{FFF9}'..='\u{FFFB}'
            | '\u{E0000}'..='\u{E007F}'
    )
}

/// Whether `text` shows anything: some character that is not whitespace, a
/// control code or a format character. A part made only of those looks empty
/// on screen, so it must not stand in for a body that has real text.
pub fn has_visible_text(text: &str) -> bool {
    text.chars().any(|c| !c.is_whitespace() && !c.is_control() && !is_format(c))
}

/// One line of header text: controls and direction overrides removed, and a
/// line break or tab (with the folding blanks around it) becomes a single
/// blank.
pub fn single_line(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_break = false;
    for c in text.chars() {
        match c {
            '\n' | '\r' | '\t' => {
                if !in_break {
                    out.push(' ');
                }
                in_break = true;
            }
            ' ' if in_break => {}
            c if c.is_control() || is_direction_override(c) => {}
            c => {
                in_break = false;
                out.push(c);
            }
        }
    }
    out
}

#[cfg(test)]
include!("text_clean.tests.rs");
