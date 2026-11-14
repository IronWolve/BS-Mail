//! The mail engine; no window or drawing code.
//! This file only declares and re-exports the modules; later batches add the
//! rest of the module map.

pub mod account_config;
mod account_oauth;
mod account_smtp;
mod imap_session;
mod mime_render;
pub mod mock;
pub mod mock_backend;
pub mod multi_backend;
mod oauth;
mod private_fs;
pub mod real_backend;
mod search;
mod send;
mod session_state;
mod sign_in_advice;
mod text_clean;

pub use account_config::{AccountConfig, ConfigError, Security, SmtpAuth, SmtpSettings, Trust};
pub use account_oauth::{AuthKind, OAuthSettings};
pub use mock::{Fixture, FixtureError};
pub use mock_backend::MockBackend;
pub use multi_backend::{Member, MembersHandle, MultiBackend};
pub use oauth::{BrowserSignIn, HttpLimits, SignInLimits, SignInPoll, TokenError, TokenSource};
pub use real_backend::RealBackend;
