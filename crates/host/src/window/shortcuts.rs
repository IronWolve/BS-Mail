//! The keys the window keeps for itself: full screen, paste, and the text
//! size. They never reach the screen code.
//!
//! The rules read the modifiers the input boundary reports, where Command on
//! a Mac already counts as Ctrl, so one table serves every platform.

use surface::Modifiers;
use winit::keyboard::{Key, NamedKey};

/// What the window does for one of its own keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shortcut {
    FullScreen,
    /// The system clipboard's text goes to the screen as a paste.
    Paste,
    /// One step up or down in the text size, or back to the starting size.
    Larger,
    Smaller,
    ResetSize,
}

/// The window's own shortcut for a key press, or `None` when the key belongs
/// to the screen.
pub fn shortcut_for(key: &Key, modifiers: Modifiers) -> Option<Shortcut> {
    match key {
        Key::Named(NamedKey::F11) => Some(Shortcut::FullScreen),
        Key::Named(NamedKey::Insert) if modifiers.shift => Some(Shortcut::Paste),
        Key::Character(text) if modifiers.ctrl => ctrl_shortcut(text.as_str(), modifiers),
        _ => None,
    }
}

fn ctrl_shortcut(text: &str, modifiers: Modifiers) -> Option<Shortcut> {
    match text {
        "v" | "V" if !modifiers.alt => Some(Shortcut::Paste),
        "=" | "+" => Some(Shortcut::Larger),
        "-" => Some(Shortcut::Smaller),
        "0" => Some(Shortcut::ResetSize),
        _ => None,
    }
}

#[cfg(test)]
include!("shortcuts.tests.rs");
