//! How the program decides whether it is a window or a terminal this time.

pub mod console_check;
pub mod decide;
pub mod display_check;

pub use decide::{Launch, decide};
