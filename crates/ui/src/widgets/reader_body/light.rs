//! The light on a link: the one under the pointer and the one the keys chose.
//! Every piece of a link, on every row in view, is restyled in place once the
//! body is drawn and before the marks are laid over it, so the selection and the
//! find win over it. Only colour and weight change: no cell is added, moved or
//! wrapped again, and the underline stays in every state.

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
    /// The keys are on it (`l` and `L`).
    Focus,
}

/// The links to light in one drawing.
#[derive(Debug, Clone, Copy)]
pub struct LinkLights<'a> {
    /// The lit links by their place among the body's links, the one that wins
    /// first: a link that is in the list twice looks as its first entry says,
    /// so the pointer's light goes before the keyboard's.
    pub lit: &'a [(usize, LinkState)],
    /// The terminal shows no colour at all, so its host drops every colour and
    /// only weight, underline and reverse video are left to tell the states
    /// apart.
    pub colourless: bool,
}

/// The style of a lit link. A theme written before these roles existed does
/// not have them, and asking a theme for a role it lacks is logged as unknown,
/// so the link then keeps its own colours and goes bold. The underline is
/// always kept.
///
/// With no colour the blocks are gone, so the states are told apart by weight
/// and reverse video: the pointer's light is bold, the button down on it is
/// reversed and bold, and the keys' is reversed and not bold.
fn light_style(ctx: &DrawContext<'_>, state: LinkState, colourless: bool) -> Style {
    let wanted: &[&str] = match state {
        LinkState::Hover => &[roles::BODY_LINK_HOVER],
        LinkState::Pressed => &[roles::BODY_LINK_PRESSED, roles::BODY_LINK_HOVER],
        LinkState::Focus => &[roles::BODY_LINK_FOCUS, roles::BODY_LINK_HOVER],
    };
    let style = match wanted.iter().find(|role| ctx.theme.has_role(role)) {
        Some(role) => ctx.full(role),
        None => ctx.fg(roles::BODY_LINK).add_modifier(Modifier::BOLD),
    }
    .add_modifier(Modifier::UNDERLINED);
    match (colourless, state) {
        (false, _) => style,
        (true, LinkState::Hover) => style.add_modifier(Modifier::BOLD),
        (true, LinkState::Pressed) => style.add_modifier(Modifier::BOLD | Modifier::REVERSED),
        (true, LinkState::Focus) => style.remove_modifier(Modifier::BOLD).add_modifier(Modifier::REVERSED),
    }
}

/// Lights the links `lights` names in the body `text` drawn in `rect`: each of
/// their pieces in the rows in view, except the cells `marks` cover.
pub fn draw_link_light(frame: &mut Frame, ctx: &DrawContext<'_>, text: &BodyText, rect: Rect, lights: &LinkLights<'_>, marks: &Highlights<'_>) {
    let styles: Vec<(usize, Style)> = lights.lit.iter().map(|(index, state)| (*index, light_style(ctx, *state, lights.colourless))).collect();
    let text_x = rect.x.saturating_add(TEXT_AT);
    let clip_right = text_x.saturating_add(wrap_width(ctx, rect)).min(rect.right());
    let last = (text.first() + text.visible()).min(text.len());
    for row in text.first()..last {
        let y = rect.y.saturating_add(u16::try_from(row - text.first()).unwrap_or(u16::MAX));
        let covered = marks.covered(text, row);
        let mut x = text_x;
        for piece in text.pieces(row) {
            let end = x.saturating_add(cells(&piece.text));
            let style = piece.link.and_then(|link| styles.iter().find(|(index, _)| *index == link)).map(|(_, style)| *style);
            if let Some(style) = style {
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
