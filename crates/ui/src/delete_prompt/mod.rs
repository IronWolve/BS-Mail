//! The question that comes before mail is deleted for good: how many messages,
//! that it cannot be undone, and two buttons with "Keep them" as the safe one.
//! This file only declares and re-exports the modules.

pub mod draw;
pub mod state;

pub use state::{Choice, DeletePrompt};
