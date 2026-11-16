//! The key bar: one keycap button per action, spread across the row with a
//! bar in each gap. When the buttons do not fit, labels go first (the Options
//! label, then the others from the right), then buttons from the right, the
//! Options button before any other, never Help and Quit.

use std::borrow::Cow;

use surface::{Effect, Frame, Rect, RoundedFill};

use super::cell_text::{cells, put_text};
use super::draw_context::DrawContext;
use crate::layout::{Spread, bar_offset, inset_h, spread_items};
use crate::theme::roles;

/// Fewest cells between two buttons.
const MIN_GAP: u16 = 3;
/// Actions that are never dropped.
const KEPT: [&str; 2] = ["help", "quit"];
/// The action of the Options button.
const OPTIONS: &str = "options";

/// A button's default key text and label; the keys table of a later stage
/// replaces the key text. The default bar (`[keybar] buttons`) offers the ten
/// buttons of the reference picture; the others (Edit as new, Archive, Spam,
/// Mark all, Unmark all, Invert marks, Next unread) are there for a bar that
/// lists them.
#[derive(Debug, PartialEq, Eq)]
struct Button {
    action: &'static str,
    key: &'static str,
    label: &'static str,
}

const BUTTONS: [Button; 18] = [
    Button { action: "compose", key: "F2", label: "Compose" },
    Button { action: "reply", key: "R", label: "Reply" },
    Button { action: "reply_all", key: "Shift+R", label: "Reply All" },
    Button { action: "forward", key: "F", label: "Forward" },
    Button { action: "edit_as_new", key: "W", label: "Edit as new" },
    Button { action: "delete", key: "D", label: "Delete" },
    Button { action: "archive", key: "E", label: "Archive" },
    Button { action: "spam", key: "!", label: "Spam" },
    Button { action: "mark_all", key: "A", label: "Mark all" },
    Button { action: "unmark_all", key: "Shift+A", label: "Unmark all" },
    Button { action: "invert_marks", key: "I", label: "Invert marks" },
    Button { action: "next_unread", key: "Shift+N", label: "Next unread" },
    Button { action: "search", key: "/", label: "Search" },
    Button { action: "move", key: "M", label: "Move" },
    Button { action: "sync", key: "S", label: "Sync" },
    Button { action: "options", key: "F10", label: "Options" },
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

/// The actions with the Options button in them: when the list does not name
/// it, it goes just before Help (before Quit when there is no Help, else at
/// the end), so it is the first button to go when the bar is narrow and Help
/// and Quit stay.
fn with_options(actions: &[String]) -> Cow<'_, [String]> {
    if actions.iter().any(|action| action == OPTIONS) {
        return Cow::Borrowed(actions);
    }
    let mut list = actions.to_vec();
    let before = |name: &str| list.iter().position(|action| action == name);
    let at = before("help").or_else(|| before("quit")).unwrap_or(list.len());
    list.insert(at, OPTIONS.to_string());
    Cow::Owned(list)
}

/// The buttons for `actions` that fit in `available` cells, applying the
/// dropping rule. Unknown action names have no button and are skipped.
fn plan_buttons(actions: &[String], available: u16) -> Vec<Placed> {
    let actions = with_options(actions);
    let mut placed: Vec<Placed> = actions
        .iter()
        .filter_map(|action| button_for(action))
        .map(|button| Placed { button, label: true })
        .collect();
    while !fits(&placed, available) {
        // Labels go first, from the right; the Options label goes before them
        // all, so the other buttons keep theirs on the reference grid.
        let label = placed.iter().position(|p| p.label && p.button.action == OPTIONS);
        if let Some(index) = label.or_else(|| placed.iter().rposition(|p| p.label)) {
            placed[index].label = false;
        } else if let Some(index) = placed.iter().rposition(|p| !KEPT.contains(&p.button.action)) {
            placed.remove(index);
        } else {
            break;
        }
    }
    placed
}

/// Where the buttons of a key bar sit: the cells they spread over, which
/// buttons fit, and where each starts.
struct Row {
    area: Rect,
    placed: Vec<Placed>,
    spread: Spread,
}

fn plan_row(rect: Rect, actions: &[String]) -> Option<Row> {
    if rect.is_empty() {
        return None;
    }
    let area = inset_h(rect, 1);
    let placed = plan_buttons(actions, area.width);
    let widths: Vec<u16> = placed.iter().map(Placed::width).collect();
    let spread = spread_items(&widths, area.width, MIN_GAP)?;
    Some(Row { area, placed, spread })
}

/// The action of the button at column `col` of the key bar in `rect`, for a
/// click. Cells between buttons belong to none.
pub fn button_at(rect: Rect, actions: &[String], col: u16) -> Option<&'static str> {
    let row = plan_row(rect, actions)?;
    row.placed.iter().enumerate().find_map(|(index, button)| {
        let x = row.area.x.saturating_add(row.spread.starts[index]);
        (col >= x && col < x.saturating_add(button.width())).then_some(button.button.action)
    })
}

/// Draws the key bar into the one-row `rect` (inside the frame).
pub fn draw_key_bar(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, actions: &[String]) {
    let Some(Row { area, placed, spread }) = plan_row(rect, actions) else {
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
