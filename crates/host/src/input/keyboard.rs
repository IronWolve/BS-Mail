//! Keys and typed text.
//!
//! A key press carries what the layout calls the key, so shortcuts match the
//! letter the user sees. Text the layout produced, including dead keys and
//! compose sequences, is delivered separately and only when no modifier is
//! holding the key for a shortcut.
//!
//! Two things differ on a Mac, and both are settled here, where the window
//! library's event becomes ours. The Command key is the shortcut key, so it
//! is reported as Ctrl: Command+C, +V, +Q, +Z and Command with the wheel then
//! mean what Ctrl does elsewhere. And Option held with a key still makes the
//! character it makes (Option+2 is a trademark sign), so that character is
//! typed. Windows and Linux are as they were: there Alt held with a key is a
//! shortcut and types nothing, and a layout's AltGr arrives without Alt.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use surface::{KeyEvent, LogicalKey, Modifiers, NamedKey, PhysicalKey};
use winit::event::ElementState;
use winit::keyboard::{Key, ModifiersState, NamedKey as WinitNamed, PhysicalKey as WinitPhysical};

/// Whether Command is the shortcut key, as it is on a Mac, so it is reported
/// as Ctrl.
const COMMAND_IS_CTRL: bool = cfg!(target_os = "macos");

/// Whether Option held with a key still types the character it makes, as it
/// does on a Mac. On Windows and Linux, Alt held with a key types nothing.
const OPTION_TYPES_TEXT: bool = cfg!(target_os = "macos");

/// The modifier keys held down.
pub fn modifiers(state: ModifiersState) -> Modifiers {
    modifiers_for(state, COMMAND_IS_CTRL)
}

/// The modifier keys held down. With `command_is_ctrl`, the Command key (the
/// window library's "super") counts as Ctrl and is not reported as meta, so a
/// screen that ignores keys with meta held does not ignore a Command shortcut.
pub fn modifiers_for(state: ModifiersState, command_is_ctrl: bool) -> Modifiers {
    let command = command_is_ctrl && state.super_key();
    Modifiers {
        shift: state.shift_key(),
        ctrl: state.control_key() || command,
        alt: state.alt_key(),
        meta: state.super_key() && !command,
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
    let pressed = event.state == ElementState::Pressed;
    text_typed(pressed, event.text.as_deref(), modifiers, OPTION_TYPES_TEXT)
}

/// `typed_text` on what the event says. Ctrl and Meta always make a key a
/// shortcut. Alt does too, except where Option still types its character
/// (`option_types_text`): that character is then the text.
fn text_typed(
    pressed: bool,
    text: Option<&str>,
    modifiers: Modifiers,
    option_types_text: bool,
) -> Option<String> {
    let alt_holds_the_key = modifiers.alt && !option_types_text;
    if !pressed || modifiers.ctrl || modifiers.meta || alt_holds_the_key {
        return None;
    }
    let text = text?;
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
