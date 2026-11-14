//! A key press from the terminal as the screen's key event, and the text it
//! typed.
//!
//! A terminal reports a letter as the letter, so a printable key without
//! Ctrl or Alt is both a key and text, exactly as a window reports it: the
//! composer types the text and commands match the key.

use crossterm::event::{KeyCode, KeyEvent as TermKey, KeyEventKind};
use surface::{InputEvent, KeyEvent, LogicalKey, Modifiers, NamedKey};

use super::keys_modifiers;

/// The events one key press makes: the key, and its text when it has some.
pub fn key_events(key: &TermKey) -> Vec<InputEvent> {
    if key.kind == KeyEventKind::Release {
        return Vec::new();
    }
    let mut modifiers = keys_modifiers(key.modifiers);
    let Some(logical) = logical_key(key.code, &mut modifiers) else {
        return Vec::new();
    };
    let mut events = vec![InputEvent::Key(KeyEvent {
        key: logical,
        physical: None,
        modifiers,
        repeat: key.kind == KeyEventKind::Repeat,
    })];
    if let (LogicalKey::Char(ch), false, false, false) = (logical, modifiers.ctrl, modifiers.alt, modifiers.meta)
        && !ch.is_control()
    {
        events.push(InputEvent::Text(ch.to_string()));
    }
    events
}

/// The key as the screen names it. Shift+Tab arrives as its own code and
/// becomes Tab with Shift held. `None` for keys the screen has no use for.
fn logical_key(code: KeyCode, modifiers: &mut Modifiers) -> Option<LogicalKey> {
    let named = match code {
        KeyCode::Char(ch) => {
            // A control letter arrives lower case; keep it so Ctrl+A matches.
            let ch = if modifiers.ctrl { ch.to_ascii_lowercase() } else { ch };
            return Some(LogicalKey::Char(ch));
        }
        KeyCode::BackTab => {
            modifiers.shift = true;
            NamedKey::Tab
        }
        KeyCode::Enter => NamedKey::Enter,
        KeyCode::Esc => NamedKey::Escape,
        KeyCode::Tab => NamedKey::Tab,
        KeyCode::Backspace => NamedKey::Backspace,
        KeyCode::Delete => NamedKey::Delete,
        KeyCode::Insert => NamedKey::Insert,
        KeyCode::Home => NamedKey::Home,
        KeyCode::End => NamedKey::End,
        KeyCode::PageUp => NamedKey::PageUp,
        KeyCode::PageDown => NamedKey::PageDown,
        KeyCode::Up => NamedKey::Up,
        KeyCode::Down => NamedKey::Down,
        KeyCode::Left => NamedKey::Left,
        KeyCode::Right => NamedKey::Right,
        KeyCode::Menu => NamedKey::Menu,
        KeyCode::F(number) if (1..=12).contains(&number) => NamedKey::Function(number),
        _ => return None,
    };
    Some(LogicalKey::Named(named))
}

#[cfg(test)]
include!("keys.tests.rs");
