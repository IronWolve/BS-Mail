//! The settings the Options screen changes: where each one lives in
//! `config.toml` and the TOML value it is written as.

use model::config::IconMode;
use model::config::font::{FONT_SIZE_MAX, FONT_SIZE_MIN, LINE_SPACING_MAX, LINE_SPACING_MIN};
use toml_edit::Value;

/// One setting and the value to give it.
#[derive(Debug, Clone, PartialEq)]
pub enum Setting {
    /// `appearance.theme`: the name of a theme.
    Theme(String),
    /// `appearance.icons`: auto, nerd, unicode or ascii.
    Icons(IconMode),
    /// `font.size`: points, brought into the allowed range when written.
    FontSize(u8),
    /// `font.line_spacing`: a row-height multiplier, brought into the allowed
    /// range when written.
    LineSpacing(f32),
}

impl Setting {
    /// The table the key lives in.
    pub(super) fn table(&self) -> &'static str {
        match self {
            Self::Theme(_) | Self::Icons(_) => "appearance",
            Self::FontSize(_) | Self::LineSpacing(_) => "font",
        }
    }

    pub(super) fn key(&self) -> &'static str {
        match self {
            Self::Theme(_) => "theme",
            Self::Icons(_) => "icons",
            Self::FontSize(_) => "size",
            Self::LineSpacing(_) => "line_spacing",
        }
    }

    /// The value as it is written: a string, an integer or a float.
    pub(super) fn toml_value(&self) -> Value {
        match self {
            Self::Theme(name) => Value::from(name.as_str()),
            Self::Icons(mode) => Value::from(icon_word(*mode)),
            Self::FontSize(points) => Value::from(i64::from((*points).clamp(FONT_SIZE_MIN, FONT_SIZE_MAX))),
            Self::LineSpacing(spacing) => Value::from(spacing_number(*spacing)),
        }
    }
}

/// The lowercase name the file uses for an icon set.
fn icon_word(mode: IconMode) -> &'static str {
    match mode {
        IconMode::Auto => "auto",
        IconMode::Nerd => "nerd",
        IconMode::Unicode => "unicode",
        IconMode::Ascii => "ascii",
    }
}

/// A spacing inside its range, as the number it reads as: `1.05`, not the
/// `1.0499999523162842` that widening the `f32` would write.
fn spacing_number(spacing: f32) -> f64 {
    let inside = if spacing.is_nan() { 1.0 } else { spacing.clamp(LINE_SPACING_MIN, LINE_SPACING_MAX) };
    format!("{inside}").parse().unwrap_or(1.0)
}

#[cfg(test)]
include!("setting.tests.rs");
