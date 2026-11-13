//! The key bar: one keycap button per action, spread across the row with a
//! bar in each gap. When the buttons do not fit, labels go first, from the
//! right, then buttons from the right, never Help and Quit.

use surface::{Effect, Frame, Rect, RoundedFill};

use super::cell_text::{cells, put_text};
use super::draw_context::DrawContext;
use crate::layout::{bar_offset, inset_h, spread_items};
use crate::theme::roles;

/// Fewest cells between two buttons.
const MIN_GAP: u16 = 3;
/// Actions that are never dropped.
const KEPT: [&str; 2] = ["help", "quit"];

/// A button's default key text and label; the keys table of a later stage
/// replaces the key text.
#[derive(Debug, PartialEq, Eq)]
struct Button {
    action: &'static str,
    key: &'static str,
    label: &'static str,
}

const BUTTONS: [Button; 10] = [
    Button { action: "compose", key: "F2", label: "Compose" },
    Button { action: "reply", key: "R", label: "Reply" },
    Button { action: "reply_all", key: "Shift+R", label: "Reply All" },
    Button { action: "forward", key: "F", label: "Forward" },
    Button { action: "delete", key: "D", label: "Delete" },
    Button { action: "search", key: "/", label: "Search" },
    Button { action: "move", key: "M", label: "Move" },
    Button { action: "sync", key: "S", label: "Sync" },
    Button { action: "help", key: "?", label: "Help" },
    Button { action: "quit", key: "Q", label: "Quit" },
];

fn button_for(action: &str) -> Option<&'static Button> {
    BUTTONS.iter().find(|button| button.action == action)
}

/// The key text an action shows on its keycap, when the action has a button.
pub fn key_text(action: &str) -> Option<&'static str> {
    button_for(action).map(|button| button.key)
}

/// A button that will be drawn, with or without its label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Placed {
    button: &'static Button,
    label: bool,
}

impl Placed {
    /// Keycap cells: a space, the key, a space.
    fn cap_width(&self) -> u16 {
        cells(self.button.key) + 2
    }

    fn width(&self) -> u16 {
        if self.label {
            self.cap_width() + 1 + cells(self.button.label)
        } else {
            self.cap_width()
        }
    }
}

fn fits(placed: &[Placed], available: u16) -> bool {
    let widths: Vec<u16> = placed.iter().map(Placed::width).collect();
    spread_items(&widths, available, MIN_GAP).is_some()
}

/// The buttons for `actions` that fit in `available` cells, applying the
/// dropping rule. Unknown action names have no button and are skipped.
fn plan_buttons(actions: &[String], available: u16) -> Vec<Placed> {
    let mut placed: Vec<Placed> = actions
        .iter()
        .filter_map(|action| button_for(action))
        .map(|button| Placed { button, label: true })
        .collect();
    while !fits(&placed, available) {
        if let Some(index) = placed.iter().rposition(|p| p.label) {
            placed[index].label = false;
        } else if let Some(index) = placed.iter().rposition(|p| !KEPT.contains(&p.button.action)) {
            placed.remove(index);
        } else {
            break;
        }
    }
    placed
}

/// Draws the key bar into the one-row `rect` (inside the frame).
pub fn draw_key_bar(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, actions: &[String]) {
    if rect.is_empty() {
        return;
    }
    let area = inset_h(rect, 1);
    let placed = plan_buttons(actions, area.width);
    let widths: Vec<u16> = placed.iter().map(Placed::width).collect();
    let Some(spread) = spread_items(&widths, area.width, MIN_GAP) else {
        return;
    };
    for (index, button) in placed.iter().enumerate() {
        let x = area.x.saturating_add(spread.starts[index]);
        draw_button(frame, ctx, area, (x, rect.y), button);
        if let Some(gap) = spread.gaps.get(index) {
            let at = x.saturating_add(button.width()).saturating_add(bar_offset(*gap));
            let bar = ctx.borders.vertical.to_string();
            put_text(frame, area, at, rect.y, &bar, ctx.fg(roles::SEPARATOR));
        }
    }
}

fn draw_button(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    area: Rect,
    (x, y): (u16, u16),
    button: &Placed,
) {
    let cap = format!(" {} ", button.button.key);
    put_text(frame, area, x, y, &cap, ctx.full(roles::KEYBAR_CAP));
    let appearance = &ctx.config.appearance;
    if let (true, Some(color)) = (appearance.keycap_round, ctx.bg_color(roles::KEYBAR_CAP)) {
        frame.push_effect(Effect::RoundedFill(RoundedFill {
            area: Rect::new(x, y, button.cap_width(), 1),
            color,
            radius: appearance.corner_radius,
        }));
    }
    if button.label {
        let label_x = x.saturating_add(button.cap_width() + 1);
        put_text(frame, area, label_x, y, button.button.label, ctx.fg(roles::KEYBAR_LABEL));
    }
}

#[cfg(test)]
include!("key_bar.tests.rs");
