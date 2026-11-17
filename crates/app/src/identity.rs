//! The name, tagline, version and home page the screen shows, taken from the
//! values the build script read out of the manifest.

use ui::AppIdentity;

pub fn identity() -> AppIdentity {
    AppIdentity::new(display_name(), env!("APP_TAGLINE"), env!("CARGO_PKG_VERSION")).with_homepage(homepage())
}

pub fn display_name() -> &'static str {
    env!("APP_DISPLAY_NAME")
}

/// The display name as it appears in a file name.
pub fn slug() -> &'static str {
    env!("APP_SLUG")
}

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The address of the home page, from the manifest's `homepage`.
pub fn homepage() -> &'static str {
    env!("APP_HOMEPAGE")
}

#[cfg(test)]
include!("identity.tests.rs");
