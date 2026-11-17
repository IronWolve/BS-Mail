//! Asking for the passphrase before anything starts, when the vault is
//! protected by one. This file only declares and re-exports the modules.
//!
//! `Gate` is the prompt (the `ui` crate draws it and reads its keys; this
//! side asks the vault); `Gated` is the one event sink a run gives its host:
//! it runs the gate first and, once the vault is open, builds the screen and
//! carries on as the screen. Building the screen is what reads the accounts and
//! starts them, so nothing of that, the move of plain-text secrets included,
//! happens before the passphrase is given, and a window opens once, not twice.

mod gate;
mod gated;
mod guard;
mod keychain;
mod target;

pub use gate::Gate;
pub use gated::{Built, Facts, Gated};
pub use guard::LockOnExit;
pub use keychain::open_keychain_vault;
pub use target::vault_folder;
