//! The body as `format=flowed` text (RFC 3676): each paragraph the person
//! wrote as one line is broken at blanks into lines of at most 76
//! characters, every break marked by a blank left at the end of the line.
//!
//! - Lines end with CRLF whatever the input used.
//! - A line that starts with a blank or with `From ` gets one more blank in
//!   front (space-stuffing), which the reader takes away again.
//! - Leading `>` marks are quoting, as in a reply: they are kept, and every
//!   piece of a broken quoted line starts with the same marks.
//! - Blanks at the end of a line the person ended are removed, or the reader
//!   would join it to the next one. The signature separator `-- ` is the
//!   one exception.

/// The longest line, counting the blank that marks a soft break.
const LINE_WIDTH: usize = 76;

/// The room a quoted line keeps for text, however deep the quoting.
const MIN_TEXT_WIDTH: usize = 20;

/// The body as flowed text with CRLF line ends. Empty text gives empty text.
pub fn flow(body: &str) -> String {
    let unified = body.replace("\r\n", "\n").replace('\r', "\n");
    let text = unified.strip_suffix('\n').unwrap_or(&unified);
    if text.is_empty() && unified.is_empty() {
        return String::new();
    }
    let mut out = String::with_capacity(body.len() + body.len() / 40 + 4);
    for line in text.split('\n') {
        flow_line(line, &mut out);
    }
    out
}

fn flow_line(line: &str, out: &mut String) {
    if line == "-- " {
        out.push_str("-- \r\n");
        return;
    }
    let depth = line.chars().take_while(|c| *c == '>').count();
    let after_marks = &line[depth..];
    // The blank after the quote marks is stuffing; a blank at the start of an
    // unquoted line is the person's own and is kept.
    let text = if depth > 0 { after_marks.strip_prefix(' ').unwrap_or(after_marks) } else { after_marks };
    let text = text.trim_end_matches([' ', '\t']);
    let marks = ">".repeat(depth);
    let lead = if depth > 0 { depth + 1 } else { 0 };
    let width = LINE_WIDTH.saturating_sub(lead).max(MIN_TEXT_WIDTH);
    for (index, piece) in wrap(text, width).iter().enumerate() {
        if depth > 0 {
            out.push_str(&marks);
            if !piece.is_empty() {
                out.push(' ');
            }
        } else if needs_stuffing(piece, index) {
            out.push(' ');
        }
        out.push_str(piece);
        out.push_str("\r\n");
    }
}

/// An unquoted piece must be stuffed when a reader would take its start for
/// something else: a blank, a quote mark (only on a continuation, where the
/// person did not mean it) or `From `.
fn needs_stuffing(piece: &str, index: usize) -> bool {
    piece.starts_with(' ') || piece.starts_with("From ") || (index > 0 && piece.starts_with('>'))
}

/// Breaks `text` into pieces of at most `width` characters, each but the
/// last ending in the blank that marks the break. A word longer than a
/// piece stays whole. Text that fits is one piece.
fn wrap(text: &str, width: usize) -> Vec<String> {
    if text.chars().count() <= width {
        return vec![text.to_owned()];
    }
    let mut pieces = Vec::new();
    let mut current = String::new();
    let mut current_len = 0;
    for word in text.split_inclusive(' ') {
        let word_len = word.chars().count();
        if current_len > 0 && current_len + word_len > width {
            pieces.push(std::mem::take(&mut current));
            current_len = 0;
        }
        current.push_str(word);
        current_len += word_len;
    }
    pieces.push(current);
    pieces
}

#[cfg(test)]
include!("flowed.tests.rs");
