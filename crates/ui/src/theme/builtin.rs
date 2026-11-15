//! The theme embedded in the program, and the lookup for built-in theme names.

use super::loader::{ThemeError, load_theme_file};
use super::resolve::Theme;

/// The name the default theme carries and `[appearance] theme` defaults to.
pub const DEFAULT_THEME_NAME: &str = "neon-night";

/// The name of the high-contrast theme.
pub const HIGH_CONTRAST_THEME_NAME: &str = "high-contrast";

const DEFAULT_THEME_TEXT: &str = include_str!("../../../../assets/themes/default.toml");
const HIGH_CONTRAST_THEME_TEXT: &str = include_str!("../../../../assets/themes/high-contrast.toml");

/// The default theme's file text, embedded at build time.
pub fn default_theme_text() -> &'static str {
    DEFAULT_THEME_TEXT
}

/// The names of the built-in themes, the default first.
pub fn builtin_theme_names() -> &'static [&'static str] {
    &[DEFAULT_THEME_NAME, HIGH_CONTRAST_THEME_NAME]
}

/// The text of a built-in theme by name; `None` for any other name.
pub fn builtin_theme_text(name: &str) -> Option<&'static str> {
    match name {
        DEFAULT_THEME_NAME => Some(DEFAULT_THEME_TEXT),
        HIGH_CONTRAST_THEME_NAME => Some(HIGH_CONTRAST_THEME_TEXT),
        _ => None,
    }
}

/// The default theme, loaded and resolved. An error here is a defect in the
/// embedded file, which the tests rule out.
pub fn default_theme() -> Result<Theme, ThemeError> {
    let file = load_theme_file(default_theme_text(), &|name| {
        builtin_theme_text(name).map(str::to_string)
    })?;
    Theme::from_file(&file)
}

#[cfg(test)]
include!("builtin.tests.rs");
