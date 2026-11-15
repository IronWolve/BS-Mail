//! Removing characters that must never reach a screen: control codes that a
//! terminal would obey, and the characters that change how a line of text is
//! laid out. Header text and body text lose the same ones (see
//! `is_layout_control`).

/// Drops control characters other than the line break and tab, and the
/// layout controls (see `is_layout_control`). Used on body text that came
/// from the network before anything draws it or looks for links in it.
pub fn strip_controls(text: &str) -> String {
    text.chars()
        .filter(|c| *c == '\n' || *c == '\t' || !(c.is_control() || is_layout_control(*c)))
        .collect()
}

/// The characters that change how text is laid out, which a sender must not
/// be able to use: the embeddings and overrides that make later text run in
/// another direction (U+202A to U+202E), the isolates (U+2066 to U+2069), and
/// the line and paragraph separators (U+2028, U+2029) that a terminal may
/// take as line breaks. A sender can use the first two to make `invoice.exe`
/// read as `invoice.fdp`, or a link as another one. The joiners and the plain
/// direction marks (U+200C to U+200F, U+061C) are left alone: names and text
/// in Persian, Arabic and Hebrew need them.
pub fn is_layout_control(c: char) -> bool {
    matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{2028}' | '\u{2029}')
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

/// One line of header text: controls and layout controls removed, and a
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
            c if c.is_control() || is_layout_control(c) => {}
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
