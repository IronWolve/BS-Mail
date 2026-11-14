//! The folder picker that opens to move messages: a list of folders with a
//! line of text to narrow it down. This file only declares and re-exports the
//! modules.

pub mod draw;
pub mod state;

pub use state::MovePicker;
