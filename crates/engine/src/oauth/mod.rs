//! OAuth2 sign-in: the browser round trip that gets the first tokens, the
//! vault entries that keep them, and the token source that hands a fresh access
//! token to the IMAP and SMTP sign-in (the design is in plan section 9.1a).
//! This file only declares and re-exports the modules.

mod authorize_url;
mod browser_sign_in;
mod error;
mod http;
mod http_parse;
mod json;
pub mod legacy_attempt;
mod loopback_listener;
mod pkce;
pub mod previous_token;
pub mod provider_table;
mod token_endpoint;
pub mod token_file;
mod token_place;
mod token_source;
pub mod token_store;
mod unsaved_tokens;
mod xoauth2;

pub use browser_sign_in::{BrowserSignIn, SignInLimits, SignInPoll};
pub use error::TokenError;
pub use http::{HttpLimits, Url};
pub use token_place::TokenPlace;
pub use token_source::TokenSource;
pub use token_store::{StoreError, TokenRecord};
pub use xoauth2::{blob as xoauth2_blob, raw as xoauth2_raw};

#[cfg(test)]
include!("fake_endpoint.tests.rs");
