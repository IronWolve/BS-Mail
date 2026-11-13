//! The mail engine; no window or drawing code.
//! This file only declares and re-exports the modules; later batches add the
//! rest of the module map.

pub mod mock;
pub mod mock_backend;

pub use mock::{Fixture, FixtureError};
pub use mock_backend::MockBackend;
