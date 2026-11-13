//! Keys and typed text.
//!
//! A key press carries what the layout calls the key, so shortcuts match the
//! letter the user sees. Text the layout produced, including dead keys and
//! compose sequences, is delivered separately and only when no modifier is
//! holding the key for a shortcut.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use surface::{KeyEvent, LogicalKey, Modifiers, NamedKey, PhysicalKey};
use winit::event::ElementState;
use winit::keyboard::{Key, ModifiersState, NamedKey as WinitNamed, PhysicalKey as WinitPhysical};

/// The modifier keys held down.
pub fn modifiers(state: ModifiersState) -> Modifiers {
    Modifiers {
        shift: state.shift_key(),
        ctrl: state.control_key(),
        alt: state.alt_key(),
        meta: state.super_key(),
    }
}

/// One key press. Releases are not passed on.
pub fn key_event(event: &winit::event::KeyEvent, modifiers: Modifiers) -> Option<KeyEvent> {
    if event.state != ElementState::Pressed {
        return None;
    }
    Some(KeyEvent {
        key: logical_key(&event.logical_key)?,
        physical: physical_key(event.physical_key),
        modifiers,
        repeat: event.repeat,
    })
}

/// The text a key press produced, when it is text and not a shortcut.
pub fn typed_text(event: &winit::event::KeyEvent, modifiers: Modifiers) -> Option<String> {
    if event.state != ElementState::Pressed || modifiers.ctrl || modifiers.alt || modifiers.meta {
        return None;
    }
    let text = event.text.as_ref()?;
    if text.chars().any(|ch| ch.is_control()) {
        return None;
    }
    Some(text.to_string())
}

/// The key as the layout names it.
fn logical_key(key: &Key) -> Option<LogicalKey> {
    match key {
        Key::Character(text) => text.chars().next().map(LogicalKey::Char),
        Key::Named(WinitNamed::Space) => Some(LogicalKey::Char(' ')),
        Key::Named(named) => named_key(*named).map(LogicalKey::Named),
        _ => None,
    }
}

/// The keys that have no character of their own.
fn named_key(named: WinitNamed) -> Option<NamedKey> {
    let key = match named {
        WinitNamed::Enter => NamedKey::Enter,
        WinitNamed::Escape => NamedKey::Escape,
        WinitNamed::Tab => NamedKey::Tab,
        WinitNamed::Backspace => NamedKey::Backspace,
        WinitNamed::Delete => NamedKey::Delete,
        WinitNamed::Insert => NamedKey::Insert,
        WinitNamed::Home => NamedKey::Home,
        WinitNamed::End => NamedKey::End,
        WinitNamed::PageUp => NamedKey::PageUp,
        WinitNamed::PageDown => NamedKey::PageDown,
        WinitNamed::ArrowUp => NamedKey::Up,
        WinitNamed::ArrowDown => NamedKey::Down,
        WinitNamed::ArrowLeft => NamedKey::Left,
        WinitNamed::ArrowRight => NamedKey::Right,
        WinitNamed::ContextMenu => NamedKey::Menu,
        other => return function_key(other).map(NamedKey::Function),
    };
    Some(key)
}

/// `F1` through `F12`.
fn function_key(named: WinitNamed) -> Option<u8> {
    let number = match named {
        WinitNamed::F1 => 1,
        WinitNamed::F2 => 2,
        WinitNamed::F3 => 3,
        WinitNamed::F4 => 4,
        WinitNamed::F5 => 5,
        WinitNamed::F6 => 6,
        WinitNamed::F7 => 7,
        WinitNamed::F8 => 8,
        WinitNamed::F9 => 9,
        WinitNamed::F10 => 10,
        WinitNamed::F11 => 11,
        WinitNamed::F12 => 12,
        _ => return None,
    };
    Some(number)
}

/// The key's place on the keyboard, as a code of our own that only has to be
/// the same for the same key.
fn physical_key(key: WinitPhysical) -> Option<PhysicalKey> {
    match key {
        WinitPhysical::Code(code) => {
            let mut hasher = DefaultHasher::new();
            code.hash(&mut hasher);
            Some(PhysicalKey(hasher.finish() as u32))
        }
        _ => None,
    }
}

#[cfg(test)]
include!("keyboard.tests.rs");
