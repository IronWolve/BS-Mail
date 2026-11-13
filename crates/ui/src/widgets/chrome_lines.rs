//! The screen borders, the dividers between regions and the rules under
//! titles, drawn from the layout's line list with the border set's pieces.
//! Where lines meet, the cell gets the right corner, tee or cross.

use surface::{Frame, Rect};

use super::cell_text::put_text;
use super::draw_context::DrawContext;
use crate::borders::LineMap;
use crate::layout::{Layout, LineKind, LineSpec, Orientation};
use crate::theme::roles;

fn add(map: &mut LineMap, line: &LineSpec) {
    match line.orientation {
        Orientation::Horizontal => map.horizontal(line.fixed, line.from, line.to),
        Orientation::Vertical => map.vertical(line.fixed, line.from, line.to),
    }
}

/// Draws every frame line and rule of `layout`.
pub fn draw_chrome_lines(frame: &mut Frame, ctx: &DrawContext<'_>, layout: &Layout) {
    let (cols, rows) = (layout.grid.cols, layout.grid.rows);
    let mut frame_lines = LineMap::new(cols, rows);
    let mut rules = LineMap::new(cols, rows);
    for line in &layout.lines {
        match line.kind {
            LineKind::Frame => add(&mut frame_lines, line),
            LineKind::Rule => add(&mut rules, line),
        }
    }
    let area = Rect::new(0, 0, cols, rows);
    let border = ctx.fg(roles::BORDER);
    let rule = ctx.fg(roles::BORDER_RULE);
    for y in 0..rows {
        for x in 0..cols {
            let (piece, style) = match frame_lines.piece_at(ctx.borders, x, y) {
                Some(piece) => (piece, border),
                None => match rules.piece_at(ctx.borders, x, y) {
                    Some(piece) => (piece, rule),
                    None => continue,
                },
            };
            put_text(frame, area, x, y, &piece.to_string(), style);
        }
    }
}

#[cfg(test)]
include!("chrome_lines.tests.rs");
