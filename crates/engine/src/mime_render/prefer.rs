//! Which body part the reading pane prefers when a message has both a plain
//! text part and an HTML part. It is the `reader.prefer` setting: one value
//! for the whole run, set by the app at start and again when the settings are
//! read again, and looked up when a body is rendered.

use std::sync::atomic::{AtomicBool, Ordering};

use model::config::BodyPreference;

static PREFER_HTML: AtomicBool = AtomicBool::new(false);

/// Takes `prefer` as the choice for every body rendered from now on. A body
/// already on screen is not drawn again.
pub fn set_body_preference(prefer: BodyPreference) {
    PREFER_HTML.store(prefer == BodyPreference::Html, Ordering::Relaxed);
}

/// The choice in force.
pub fn body_preference() -> BodyPreference {
    if PREFER_HTML.load(Ordering::Relaxed) { BodyPreference::Html } else { BodyPreference::Plain }
}

#[cfg(test)]
include!("prefer.tests.rs");
