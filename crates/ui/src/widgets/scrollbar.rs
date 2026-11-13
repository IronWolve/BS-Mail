//! A one-column scrollbar: a track the height of the region and a thumb the
//! size of the visible share, drawn only when the content overflows.

use surface::{Frame, Rect};

use super::cell_text::put_text;
use super::draw_context::DrawContext;
use crate::icons::IconKind;
use crate::theme::roles;

/// The thumb glyph in the icon-font and Unicode sets.
const THUMB: &str = "\u{2588}";
/// The plain-text sets: thumb and track.
const ASCII_THUMB: &str = "#";
const ASCII_TRACK: &str = "|";

/// Rows (from the top of the track) where the thumb starts and how many it
/// covers, for `first` of `total` rows with `visible` shown in `height` rows.
fn thumb_span(first: usize, visible: usize, total: usize, height: usize) -> (usize, usize) {
    let size = (visible * height / total).clamp(1, height);
    let scrollable = total - visible;
    let first = first.min(scrollable);
    let top = (first * (height - size) + scrollable / 2) / scrollable;
    (top, size)
}

/// Draws the bar in column `col` over the rows of `rows`. Nothing is drawn
/// when all `total` rows fit in `visible`, or the region has no height.
pub fn draw_scrollbar(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    col: u16,
    rows: Rect,
    first: usize,
    visible: usize,
    total: usize,
) {
    if total <= visible || visible == 0 || rows.height == 0 {
        return;
    }
    let ascii = ctx.icons.kind() == IconKind::Ascii;
    let track = if ascii { ASCII_TRACK.to_string() } else { ctx.borders.vertical.to_string() };
    let thumb = if ascii { ASCII_THUMB } else { THUMB };
    let (top, size) = thumb_span(first, visible, total, usize::from(rows.height));
    let clip = Rect::new(col, rows.y, 1, rows.height);
    for offset in 0..usize::from(rows.height) {
        let y = rows.y.saturating_add(u16::try_from(offset).unwrap_or(u16::MAX));
        let (glyph, role) = if (top..top + size).contains(&offset) {
            (thumb, roles::SCROLLBAR_THUMB)
        } else {
            (track.as_str(), roles::SCROLLBAR_TRACK)
        };
        put_text(frame, clip, col, y, glyph, ctx.fg(role));
    }
}

#[cfg(test)]
include!("scrollbar.tests.rs");
