//! The question that comes before any delete: how many messages move to the
//! Trash or go for good (which cannot be undone), and two buttons with "Keep
//! them" as the safe one. This file only declares and re-exports the modules.

pub mod draw;
pub mod state;

pub use state::{Choice, DeletePrompt};
