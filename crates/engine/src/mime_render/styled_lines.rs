//! Body text to styled lines. Only the roles the reading pane does not work
//! out for itself are set here: plain text, quotes and the signature block.
//! Bullets, links, paths and the like are the pane's job.

use model::{BodyRole, StyledLine, StyledSpan};

use crate::text_clean::strip_controls;

/// Tab stops are this many cells apart.
const TAB: usize = 8;

/// One line per text line. Trailing blank lines are dropped; a line that
/// starts with `>` is a quote; everything after a `-- ` line is signature.
pub fn style_lines(text: &str) -> Vec<StyledLine> {
    let clean = strip_controls(text);
    let mut lines: Vec<&str> = clean.split('\n').map(trim_cr).collect();
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }
    let mut in_signature = false;
    lines
        .into_iter()
        .map(|line| {
            let role = if in_signature {
                BodyRole::Signature
            } else if is_signature_mark(line) {
                in_signature = true;
                BodyRole::Signature
            } else if line.trim_start().starts_with('>') {
                BodyRole::Quote
            } else {
                BodyRole::Text
            };
            styled(line, role)
        })
        .collect()
}

fn trim_cr(line: &str) -> &str {
    line.strip_suffix('\r').unwrap_or(line)
}

/// The signature delimiter: two dashes and a blank, or two dashes alone when
/// the sender's program trimmed the blank.
fn is_signature_mark(line: &str) -> bool {
    line == "-- " || line == "--"
}

fn styled(line: &str, role: BodyRole) -> StyledLine {
    if line.is_empty() {
        return StyledLine::default();
    }
    StyledLine {
        spans: vec![StyledSpan {
            text: expand_tabs(line),
            role,
        }],
    }
}

fn expand_tabs(line: &str) -> String {
    if !line.contains('\t') {
        return line.to_owned();
    }
    let mut out = String::with_capacity(line.len() + TAB);
    let mut column = 0;
    for c in line.chars() {
        if c == '\t' {
            let pad = TAB - column % TAB;
            out.extend(std::iter::repeat_n(' ', pad));
            column += pad;
        } else {
            out.push(c);
            column += 1;
        }
    }
    out
}

#[cfg(test)]
include!("styled_lines.tests.rs");
