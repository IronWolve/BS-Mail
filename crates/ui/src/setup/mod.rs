//! The first-run screen that takes an email account: the form's state, the
//! server details guessed from the address, its keys and its drawing. This
//! file only declares and re-exports the modules.

pub mod draw;
pub mod form;
pub mod input;
pub mod providers;

pub use form::{Security, SetupField, SetupForm};
