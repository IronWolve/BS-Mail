//! Terminal input turned into the events the screen reads. This file only
//! declares and re-exports the modules.

pub mod events;
pub mod keys;
pub mod mouse;

pub use events::translate;

use crossterm::event::KeyModifiers;
use surface::Modifiers;

/// The modifiers held, as the screen names them.
pub(crate) fn keys_modifiers(held: KeyModifiers) -> Modifiers {
    Modifiers {
        shift: held.contains(KeyModifiers::SHIFT),
        ctrl: held.contains(KeyModifiers::CONTROL),
        alt: held.contains(KeyModifiers::ALT),
        meta: held.contains(KeyModifiers::SUPER) || held.contains(KeyModifiers::META),
    }
}
