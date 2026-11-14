//! The mail engine; no window or drawing code.
//! This file only declares and re-exports the modules; later batches add the
//! rest of the module map.

pub mod account_config;
mod account_smtp;
mod imap_session;
mod mime_render;
pub mod mock;
pub mod mock_backend;
pub mod real_backend;
mod send;
mod session_state;
mod text_clean;

pub use account_config::{AccountConfig, ConfigError, Security, SmtpSettings, Trust};
pub use mock::{Fixture, FixtureError};
pub use mock_backend::MockBackend;
pub use real_backend::RealBackend;
