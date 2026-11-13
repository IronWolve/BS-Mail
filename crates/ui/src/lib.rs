//! Screen drawing and input handling; no window, no network, no database.
//! This file only declares and re-exports the modules; later batches add the
//! rest of the module map.

pub mod app_identity;
pub mod app_view;
pub mod borders;
pub mod format;
pub mod icons;
pub mod input;
pub mod layout;
pub mod theme;
pub mod view_state;
pub mod widgets;

pub use app_identity::AppIdentity;
pub use app_view::AppView;
