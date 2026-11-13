//! Removing characters that must never reach a screen: control codes that a
//! terminal would obey.

/// Drops control characters other than the line break and tab. Used on text
/// that came from the network before anything draws it.
pub fn strip_controls(text: &str) -> String {
    text.chars()
        .filter(|c| *c == '\n' || *c == '\t' || !c.is_control())
        .collect()
}

/// One line of header text: controls removed, and a line break or tab (with
/// the folding blanks around it) becomes a single blank.
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
            c if c.is_control() => {}
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
