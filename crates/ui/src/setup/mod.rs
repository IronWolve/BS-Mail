//! The first-run screen that takes an email account: the form's state, the
//! server details guessed from the address, its keys and its drawing. This
//! file only declares and re-exports the modules.

pub mod account_id;
pub mod action;
pub mod ascii_domain;
pub mod draw;
pub mod fields;
pub mod form;
pub mod form_debug;
pub mod form_toml;
pub mod input;
pub mod layout;
pub mod local_host;
pub mod link_panel;
pub mod link_panel_input;
pub mod prefill;
pub mod providers;
pub mod saved_client;
pub mod simple_providers;
pub mod trust_note;

pub use account_id::account_id_for;
pub use action::SetupAction;
pub use fields::{Security, SetupField, SignIn};
pub use form::SetupForm;
pub use prefill::AccountPrefill;
pub use saved_client::SavedClient;
