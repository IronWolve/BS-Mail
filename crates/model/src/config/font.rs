//! The `[font]` table: font file, size and line spacing (window mode only).

use serde::{Deserialize, Serialize};

pub const FONT_SIZE_MIN: u8 = 6;
pub const FONT_SIZE_MAX: u8 = 40;
pub const LINE_SPACING_MIN: f32 = 0.9;
pub const LINE_SPACING_MAX: f32 = 1.6;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FontConfig {
    /// Empty means the embedded default; else a name in the data directory's
    /// fonts folder or an absolute path. Not built yet: the key is accepted
    /// and the embedded font is always used.
    pub file: String,
    /// Points, from `FONT_SIZE_MIN` to `FONT_SIZE_MAX`.
    pub size: u8,
    /// Multiplies the row height, from `LINE_SPACING_MIN` to `LINE_SPACING_MAX`.
    pub line_spacing: f32,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            file: String::new(),
            size: 12,
            line_spacing: 1.0,
        }
    }
}
