//! OAuth2 sign-in: the browser round trip that gets the first tokens, the
//! token file that keeps them, and the token source that hands a fresh access
//! token to the IMAP and SMTP sign-in (the design is in plan section 9.1a).
//! This file only declares and re-exports the modules.

mod authorize_url;
mod browser_sign_in;
mod error;
mod http;
mod http_parse;
mod json;
mod loopback_listener;
mod pkce;
pub mod provider_table;
mod token_endpoint;
mod token_file;
mod token_source;
mod xoauth2;

pub use browser_sign_in::{BrowserSignIn, SignInLimits, SignInPoll};
pub use error::TokenError;
pub use http::{HttpLimits, Url};
pub use token_source::TokenSource;
pub use xoauth2::{blob as xoauth2_blob, raw as xoauth2_raw};

#[cfg(test)]
include!("fake_endpoint.tests.rs");
