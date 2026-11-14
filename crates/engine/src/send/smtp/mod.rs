//! Submitting a message to the account's mail server over SMTP: one
//! connection per send, TLS before any password, a time limit on every step.
//! This file only declares and re-exports the modules.

mod auth;
mod caps;
mod connect;
mod data;
mod envelope;
mod error;
mod link;
mod reply;
mod submit;
mod timeouts;
mod wire;

pub use error::SmtpError;
pub use submit::submit;
pub use timeouts::SmtpTimeouts;
