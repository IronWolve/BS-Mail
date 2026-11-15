//! `format=flowed` text (RFC 3676): paragraphs the sender broke with a
//! trailing blank are joined back into one line each. Where to wrap them is
//! the reader's business.

/// Rebuilds the paragraphs. `delsp` says the blank that marks a soft break is
/// not part of the text. Quote depth is kept as `>` marks; a quoted
/// paragraph is only joined with lines of the same depth.
pub fn unflow(text: &str, delsp: bool) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut current: Option<(usize, String)> = None;
    for raw in text.split('\n') {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let (depth, content) = split_quote(line);
        // The signature separator is a fixed line at any quote depth (RFC 3676 4.3).
        let soft = content.ends_with(' ') && content != "-- ";
        if !matches!(&current, Some((open, _)) if *open == depth) {
            flush(current.take(), &mut out);
            current = Some((depth, String::new()));
        }
        if let Some((_, paragraph)) = current.as_mut() {
            let piece = if soft && delsp {
                &content[..content.len() - 1]
            } else {
                content
            };
            paragraph.push_str(piece);
        }
        if !soft {
            flush(current.take(), &mut out);
        }
    }
    flush(current.take(), &mut out);
    out.join("\n")
}

/// The quote depth of a line and its text with the quote marks and the one
/// stuffed blank removed.
fn split_quote(line: &str) -> (usize, &str) {
    let depth = line.chars().take_while(|c| *c == '>').count();
    let rest = &line[depth..];
    (depth, rest.strip_prefix(' ').unwrap_or(rest))
}

fn flush(paragraph: Option<(usize, String)>, out: &mut Vec<String>) {
    if let Some((depth, text)) = paragraph {
        let marks = ">".repeat(depth);
        let gap = if depth > 0 && !text.is_empty() { " " } else { "" };
        out.push(format!("{marks}{gap}{text}"));
    }
}

#[cfg(test)]
include!("flowed.tests.rs");
