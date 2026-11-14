//! Keys, typed text and clicks on the sign-in link page. Letters arrive as
//! text, so `o` and `c` are read there; Escape and Enter are keys.

use surface::{KeyEvent, LogicalKey, NamedKey};

use super::input::Outcome;
use super::layout::area_of;
use super::link_panel::{PanelChoice, button_at, rects};

/// A key while the page is open.
pub fn key(key: &KeyEvent) -> Outcome {
    let held = key.modifiers;
    if held.alt || held.meta || held.ctrl {
        return Outcome::Ignored;
    }
    match key.key {
        LogicalKey::Named(NamedKey::Escape) => Outcome::Panel(PanelChoice::Cancel),
        LogicalKey::Named(NamedKey::Enter) => Outcome::Panel(PanelChoice::Open),
        _ => Outcome::Ignored,
    }
}

/// Typed text while the page is open.
pub fn text(text: &str) -> Outcome {
    match text {
        "o" | "O" => Outcome::Panel(PanelChoice::Open),
        "c" | "C" => Outcome::Panel(PanelChoice::Copy),
        _ => Outcome::Ignored,
    }
}

/// A click at a cell of a screen `grid` cells large.
pub fn click(grid: (u16, u16), col: u16, row: u16) -> Outcome {
    match rects(area_of(grid)).and_then(|rects| button_at(&rects, col, row)) {
        Some(choice) => Outcome::Panel(choice),
        None => Outcome::Ignored,
    }
}

#[cfg(test)]
include!("link_panel_input.tests.rs");
