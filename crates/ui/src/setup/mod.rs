//! The first-run screen that takes an email account: the form's state, the
//! server details guessed from the address, its keys and its drawing. This
//! file only declares and re-exports the modules.

pub mod action;
pub mod draw;
pub mod fields;
pub mod form;
pub mod form_toml;
pub mod input;
pub mod layout;
pub mod link_panel;
pub mod link_panel_input;
pub mod providers;

pub use action::SetupAction;
pub use fields::{Security, SetupField, SignIn};
pub use form::SetupForm;
