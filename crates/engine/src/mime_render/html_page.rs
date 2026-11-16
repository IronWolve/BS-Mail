//! The safe copy of a message's HTML part, made where the reading of a body
//! is made: on a thread of its own, with a time limit and a catch for a panic,
//! so a message built to stall the parser stalls neither the worker nor the
//! account.

use super::guarded;
use crate::html_mail::{Outcome, build_page};

/// The safe copy of the HTML part of the message `raw`, or the words for why
/// it could not be made in time.
pub async fn render_page(raw: Vec<u8>) -> Result<Outcome, &'static str> {
    guarded::run(move || build_page(&raw)).await.map_err(|failure| failure.words())
}

#[cfg(test)]
include!("html_page.tests.rs");
