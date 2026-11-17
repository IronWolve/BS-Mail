//! The folder picker: a list of folders with a line of text to narrow it
//! down. It opens to move messages, and to go to a folder (`gg`). This file
//! only declares and re-exports the modules.

pub mod draw;
pub mod rows;
pub mod state;

pub use state::{MovePicker, Purpose};
