//! The screen with its mail source.
//!
//! Every way of showing it — a terminal, a window, a picture file — starts
//! from here, so all three show the same thing. This file only declares and
//! re-exports the modules.

mod build;
mod sources;
mod state;

pub use build::build;
pub(crate) use build::{icon_kind, plain_colors};
pub use state::SETTLE_LIMIT;
