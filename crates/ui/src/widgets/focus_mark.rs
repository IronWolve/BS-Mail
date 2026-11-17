//! The cue for which pane has the keys: its title is drawn in the accent color
//! (role `title.focused`) and underlined.
//!
//! Done after the pane is drawn, on the cells it already wrote, so no pane
//! has to know whether it is the focused one.

use surface::{Frame, Modifier, Rect};

use super::draw_context::DrawContext;
use crate::theme::{roles, surface_color};

/// Where a pane's title is, and what the pane draws it in.
#[derive(Debug, Clone, Copy)]
pub struct Heading {
    /// The title row: every written cell on it is underlined, the key hints
    /// and the counter at its ends too.
    pub row: Rect,
    /// The heading proper, the icon and the title text, without the hints and
    /// the counter: the part drawn in the accent color.
    pub text: Rect,
    /// The role the pane draws its title in. Only a cell in that color is
    /// recolored, so a tag or an alert word in a subject keeps its own.
    pub role: &'static str,
}

fn within(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.left() && col < rect.right() && row >= rect.top() && row < rect.bottom()
}

/// Marks the title of the pane that has the keys. Every written cell of the
/// row is underlined, leaving blanks alone so the line sits under the words
/// and not across the gaps; the heading is drawn in `title.focused`. A theme
/// without that role keeps the title's own colors.
///
/// A terminal with no color shows only attributes, so there the heading is
/// also bold and reversed: the underline alone would be the one sign.
pub fn mark_focused(frame: &mut Frame, ctx: &DrawContext<'_>, heading: &Heading, colourless: bool) {
    let own = ctx.fg_color(heading.role);
    let accent = ctx.theme.has_role(roles::TITLE_FOCUSED).then(|| ctx.theme.role(roles::TITLE_FOCUSED));
    let row = heading.row;
    for y in row.top()..row.bottom() {
        for x in row.left()..row.right() {
            let Some(cell) = frame.cells.cell_mut((x, y)) else {
                continue;
            };
            if cell.symbol().trim().is_empty() {
                continue;
            }
            cell.modifier.insert(Modifier::UNDERLINED);
            if !within(heading.text, x, y) {
                continue;
            }
            if let Some(style) = accent.as_ref().filter(|_| cell.fg == own) {
                cell.fg = surface_color(style.fg);
                if let Some(bg) = style.bg {
                    cell.bg = surface_color(bg);
                }
                cell.modifier.insert(style.modifiers());
            }
            if colourless {
                cell.modifier.insert(Modifier::BOLD | Modifier::REVERSED);
            }
        }
    }
}

#[cfg(test)]
include!("focus_mark.tests.rs");
