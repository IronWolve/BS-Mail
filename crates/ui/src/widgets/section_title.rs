//! A sidebar section's title row: an icon slot and the bold title.

use surface::{Frame, Rect};

use super::cell_text::{put_cut, put_text};
use super::draw_context::DrawContext;
use crate::icons::{IconId, SLOT_CELLS};
use crate::theme::roles;

/// Writes the icon at the left edge of `rect` and the title one cell after
/// the icon slot, cut to fit.
pub fn draw_section_title(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    rect: Rect,
    icon: IconId,
    title: &str,
) {
    if rect.is_empty() {
        return;
    }
    put_text(frame, rect, rect.x, rect.y, ctx.icon(icon), ctx.fg(roles::SECTION_ICON));
    let title_x = rect.x.saturating_add(SLOT_CELLS + 1);
    let room = rect.right().saturating_sub(title_x);
    put_cut(frame, rect, title_x, rect.y, title, room, ctx.fg(roles::SECTION_TITLE));
}

#[cfg(test)]
include!("section_title.tests.rs");
