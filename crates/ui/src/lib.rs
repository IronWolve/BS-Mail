//! Screen drawing and input handling; no window, no network, no database.
//! This file only declares and re-exports the modules; later batches add the
//! rest of the module map.

pub mod app_identity;
pub mod app_view;
pub mod borders;
pub mod composer;
pub mod format;
pub mod icons;
pub mod input;
pub mod layout;
pub mod delete_prompt;
pub mod move_picker;
pub mod options;
pub mod search;
pub mod setup;
pub mod theme;
pub mod view_bands;
pub mod view_facts;
pub mod view_panes;
pub mod view_sidebar;
pub mod view_state;
pub mod widgets;

pub use app_identity::AppIdentity;
pub use app_view::AppView;
pub use composer::AddressBook;
pub use options::{AccountSummary, OptionsAction, OptionsPaths};
pub use setup::{AccountPrefill, SavedClient};
