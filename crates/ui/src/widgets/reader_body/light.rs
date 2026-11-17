//! The light on a link under the pointer. Every piece of that link, on every
//! row in view, is restyled in place once the body is drawn and before the
//! marks are laid over it, so the selection and the find win over it. Only
//! colour and weight change: no cell is added, moved or wrapped again, and the
//! underline stays in every state.

use surface::{Frame, Modifier, Rect, Style};

use super::draw::{TEXT_AT, wrap_width};
use super::marks::Highlights;
use super::text_map::BodyText;
use crate::theme::roles;
use crate::widgets::DrawContext;
use crate::widgets::cell_text::cells;

/// How a lit link looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    /// The pointer is over it.
    Hover,
    /// The button is down on it and has not yet become a selection.
    Pressed,
}

/// The style of a lit link. A theme written before these roles existed does
/// not have them, and asking a theme for a role it lacks is logged as unknown,
/// so the link then keeps its own colours and goes bold. The underline is
/// always kept.
fn light_style(ctx: &DrawContext<'_>, state: LinkState) -> Style {
    let wanted: &[&str] = match state {
        LinkState::Hover => &[roles::BODY_LINK_HOVER],
        LinkState::Pressed => &[roles::BODY_LINK_PRESSED, roles::BODY_LINK_HOVER],
    };
    let style = match wanted.iter().find(|role| ctx.theme.has_role(role)) {
        Some(role) => ctx.full(role),
        None => ctx.fg(roles::BODY_LINK).add_modifier(Modifier::BOLD),
    };
    style.add_modifier(Modifier::UNDERLINED)
}

/// Lights link number `index` of the body `text` drawn in `rect`: each of its
/// pieces in the rows in view, except the cells `marks` cover.
pub fn draw_link_light(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    text: &BodyText,
    rect: Rect,
    (index, state): (usize, LinkState),
    marks: &Highlights<'_>,
) {
    let style = light_style(ctx, state);
    let text_x = rect.x.saturating_add(TEXT_AT);
    let clip_right = text_x.saturating_add(wrap_width(ctx, rect)).min(rect.right());
    let last = (text.first() + text.visible()).min(text.len());
    for row in text.first()..last {
        let y = rect.y.saturating_add(u16::try_from(row - text.first()).unwrap_or(u16::MAX));
        let covered = marks.covered(text, row);
        let mut x = text_x;
        for piece in text.pieces(row) {
            let end = x.saturating_add(cells(&piece.text));
            if piece.link == Some(index) {
                for at in x..end.min(clip_right) {
                    let col = at - text_x;
                    if covered.iter().any(|(from, to)| col >= *from && col < *to) {
                        continue;
                    }
                    if let Some(cell) = frame.cells.cell_mut((at, y)) {
                        cell.set_style(style);
                    }
                }
            }
            x = end;
        }
    }
}

#[cfg(test)]
include!("light.tests.rs");
