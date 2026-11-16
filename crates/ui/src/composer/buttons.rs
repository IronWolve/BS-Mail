//! The Send and Cancel buttons at the right end of the title row: a keycap
//! with the key, and the word.

use surface::{Effect, Frame, Rect, RoundedFill};

use super::form::{ComposeForm, Progress};
use crate::theme::roles;
use crate::widgets::{DrawContext, put_text};

/// The key shown on the Send button: the one that works in every host.
pub const SEND_KEY: &str = "F10";
pub const SEND_LABEL: &str = "Send";
pub const CANCEL_KEY: &str = "Esc";
pub const CANCEL_LABEL: &str = "Cancel";
/// Cells between the two buttons.
const GAP: u16 = 2;
/// Cells between the title and the first button.
const FROM_TITLE: u16 = 2;

/// Where the buttons sit in the title row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonRects {
    pub send: Rect,
    pub cancel: Rect,
    /// The words show beside the keys; on a narrow row only the keys do.
    pub labels: bool,
}

fn cells(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

/// A keycap is the key between blanks; the word follows after one more.
fn width(key: &str, label: &str, labels: bool) -> u16 {
    cells(key) + 2 + if labels { 1 + cells(label) } else { 0 }
}

/// The buttons in the one-row `title` area that has `title_cells` taken by the
/// title at its left; `None` when even the keys do not fit.
pub fn button_rects(title: Rect, title_cells: u16) -> Option<ButtonRects> {
    let room = title.width.saturating_sub(title_cells + FROM_TITLE);
    for labels in [true, false] {
        let send = width(SEND_KEY, SEND_LABEL, labels);
        let cancel = width(CANCEL_KEY, CANCEL_LABEL, labels);
        if send + GAP + cancel <= room {
            let cancel_at = title.right().saturating_sub(cancel);
            let send_at = cancel_at.saturating_sub(GAP + send);
            return Some(ButtonRects {
                send: Rect::new(send_at, title.y, send, 1),
                cancel: Rect::new(cancel_at, title.y, cancel, 1),
                labels,
            });
        }
    }
    None
}

/// Which button is at cell (`col`, `row`).
pub fn button_at(rects: &ButtonRects, col: u16, row: u16) -> Option<Pressed> {
    let inside = |rect: Rect| row == rect.y && col >= rect.x && col < rect.right();
    if inside(rects.send) {
        Some(Pressed::Send)
    } else if inside(rects.cancel) {
        Some(Pressed::Cancel)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pressed {
    Send,
    Cancel,
}

/// Draws both buttons; Send is dimmed while the message is with the server.
pub fn draw_buttons(frame: &mut Frame, ctx: &DrawContext<'_>, title: Rect, rects: &ButtonRects, form: &ComposeForm) {
    let sending = form.progress == Progress::Sending;
    draw_one(frame, ctx, title, rects.send, (SEND_KEY, SEND_LABEL), rects.labels, sending);
    draw_one(frame, ctx, title, rects.cancel, (CANCEL_KEY, CANCEL_LABEL), rects.labels, false);
}

pub(super) fn draw_one(frame: &mut Frame, ctx: &DrawContext<'_>, clip: Rect, rect: Rect, (key, label): (&str, &str), labels: bool, dim: bool) {
    let cap_cells = cells(key) + 2;
    let cap_style = if dim { ctx.full(roles::KEYBAR_CAP).add_modifier(surface::Modifier::DIM) } else { ctx.full(roles::KEYBAR_CAP) };
    put_text(frame, clip, rect.x, rect.y, &format!(" {key} "), cap_style);
    let appearance = &ctx.config.appearance;
    if let (true, Some(color), false) = (appearance.keycap_round, ctx.bg_color(roles::KEYBAR_CAP), dim) {
        frame.push_effect(Effect::RoundedFill(RoundedFill {
            area: Rect::new(rect.x, rect.y, cap_cells, 1),
            color,
            radius: appearance.corner_radius,
        }));
    }
    if labels {
        let style = if dim { ctx.fg(roles::KEYBAR_LABEL).add_modifier(surface::Modifier::DIM) } else { ctx.fg(roles::KEYBAR_LABEL) };
        put_text(frame, clip, rect.x + cap_cells + 1, rect.y, label, style);
    }
}

#[cfg(test)]
include!("buttons.tests.rs");
