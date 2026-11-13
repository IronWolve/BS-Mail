//! A row of segments spread with equal gaps and a separator bar centered in
//! each gap (the account strip and the footer).

use surface::{Frame, Rect};

use super::cell_text::{Run, put_runs, put_text, runs_width};
use super::draw_context::DrawContext;
use crate::layout::{bar_offset, spread_items};
use crate::theme::roles;

/// Fewest cells between two segments.
pub const MIN_GAP: u16 = 3;

/// True when the segments fit in `width` cells with the minimum gap.
pub fn segments_fit(segments: &[Vec<Run>], width: u16) -> bool {
    let widths: Vec<u16> = segments.iter().map(|runs| runs_width(runs)).collect();
    spread_items(&widths, width, MIN_GAP).is_some()
}

/// Draws the segments on `row` across `area`: the first starts at the left
/// edge and the last ends at the right edge. Returns false and draws nothing
/// when they do not fit; the caller drops a segment and tries again.
pub fn draw_segments(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    area: Rect,
    row: u16,
    segments: &[Vec<Run>],
) -> bool {
    let widths: Vec<u16> = segments.iter().map(|runs| runs_width(runs)).collect();
    let Some(spread) = spread_items(&widths, area.width, MIN_GAP) else {
        return false;
    };
    let bar = ctx.borders.vertical.to_string();
    let bar_style = ctx.fg(roles::SEPARATOR);
    for (index, runs) in segments.iter().enumerate() {
        let x = area.x.saturating_add(spread.starts[index]);
        put_runs(frame, area, x, row, runs);
        if let Some(gap) = spread.gaps.get(index) {
            let gap_start = x.saturating_add(widths[index]);
            let at = gap_start.saturating_add(bar_offset(*gap));
            put_text(frame, area, at, row, &bar, bar_style);
        }
    }
    true
}

#[cfg(test)]
include!("segment_row.tests.rs");
