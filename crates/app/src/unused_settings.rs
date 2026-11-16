//! Settings the file takes that nothing reads yet.
//!
//! Each of these is a valid key with a default that matches what the program
//! does today. Set to anything else it would change nothing, so the load says
//! so once, in one line, instead of leaving the user to wonder.

use model::config::Config;

/// The settings in `config` that are set away from their default and that
/// nothing reads yet, as `table.key`.
pub fn unused(config: &Config) -> Vec<&'static str> {
    let default = Config::default();
    let mut names = Vec::new();
    // Only the embedded font is loaded.
    if config.font.file != default.font.file {
        names.push("font.file");
    }
    // Both borders draw the same frames.
    if config.appearance.pane_borders != default.appearance.pane_borders {
        names.push("appearance.pane_borders");
    }
    names
}

/// The line to show for `config`, when something in it is not used yet.
pub fn note(config: &Config) -> Option<String> {
    let names = unused(config);
    if names.is_empty() {
        return None;
    }
    Some(format!("not built yet, so these settings change nothing: {}", names.join(", ")))
}

#[cfg(test)]
include!("unused_settings.tests.rs");
