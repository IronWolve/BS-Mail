//! The name, tagline and version the screen shows, taken from the values the
//! build script read out of the manifest.

use ui::AppIdentity;

pub fn identity() -> AppIdentity {
    AppIdentity::new(display_name(), env!("APP_TAGLINE"), env!("CARGO_PKG_VERSION"))
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
