//! The in-app Options screen: pages for accounts, appearance and data. The
//! screen only shows facts the program gives it and queues requests the
//! program takes; it never touches a file or a server. This file only
//! declares and re-exports the modules.

pub mod accounts_page;
pub mod action;
pub mod appearance_page;
pub mod appearance_values;
pub mod confirm_page;
pub mod control;
pub mod data_page;
pub mod draw;
pub mod draw_rows;
pub mod empty_hint;
pub mod host;
pub mod input;
pub mod layout;
pub mod layout_choices;
pub mod layout_page;
pub mod mouse;
pub mod pages;
pub mod place;
pub mod rows;
pub mod state;
pub mod summary;
pub mod view;
pub mod view_api;

pub use action::OptionsAction;
pub use control::{Control, Part};
pub use layout_choices::{Column, LayoutChange, Panel};
pub use state::{OptionsState, Page, Stop};
pub use summary::{AccountSummary, OptionsPaths};
