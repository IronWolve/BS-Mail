//! Soft wrapping: how a logical line is cut into the lines that show on
//! screen, and where the cursor sits among them.
//!
//! A line breaks after the last space that fits, or in the middle of a word
//! when one word is wider than the whole width. The space stays at the end of
//! the line it ends, so no cell is lost and every grapheme belongs to exactly
//! one visual line.

use super::graphemes::{is_blank, list, width};

/// One screen line: graphemes `start` up to `end` of logical line `line`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisualLine {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

/// Cuts every logical line to `width` cells. An empty line is one empty
/// visual line, so it can hold the cursor.
pub fn wrap(lines: &[String], width: usize) -> Vec<VisualLine> {
    let width = width.max(1);
    let mut out = Vec::new();
    for (index, text) in lines.iter().enumerate() {
        wrap_line(index, text, width, &mut out);
    }
    out
}

fn wrap_line(index: usize, text: &str, room: usize, out: &mut Vec<VisualLine>) {
    let graphemes = list(text);
    if graphemes.is_empty() {
        out.push(VisualLine { line: index, start: 0, end: 0 });
        return;
    }
    let mut start = 0;
    while start < graphemes.len() {
        let end = break_point(&graphemes, start, room);
        out.push(VisualLine { line: index, start, end });
        start = end;
    }
}

/// Where the line that starts at `start` ends.
fn break_point(graphemes: &[&str], start: usize, room: usize) -> usize {
    let mut used = 0;
    let mut last_blank = None;
    for (offset, grapheme) in graphemes[start..].iter().enumerate() {
        let cells = width(grapheme);
        if used + cells > room {
            // Too wide for this line: break after the last blank, if any is
            // after the first cell, else here, always taking at least one.
            return match last_blank {
                Some(blank) => blank + 1,
                None => (start + offset).max(start + 1),
            };
        }
        used += cells;
        if is_blank(grapheme) && offset > 0 {
            last_blank = Some(start + offset);
        }
    }
    graphemes.len()
}

/// The visual line holding the cursor at (`line`, `col`). At the boundary
/// between two wrapped lines the cursor belongs to the start of the second.
pub fn visual_index(visual: &[VisualLine], line: usize, col: usize) -> usize {
    let mut found = 0;
    for (at, piece) in visual.iter().enumerate() {
        if piece.line != line {
            continue;
        }
        found = at;
        if col < piece.end || (col == piece.end && is_last_of_line(visual, at)) {
            return at;
        }
    }
    found
}

fn is_last_of_line(visual: &[VisualLine], at: usize) -> bool {
    visual.get(at + 1).is_none_or(|next| next.line != visual[at].line)
}

#[cfg(test)]
include!("wrap.tests.rs");
