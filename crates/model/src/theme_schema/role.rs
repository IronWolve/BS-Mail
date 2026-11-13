//! A theme role: optional colors and modifiers, plus references to palette colors.

use serde::{Deserialize, Serialize};

use super::palette::Rgb;

/// The role every other role inherits its omitted fields from.
pub const BASE_ROLE: &str = "base";

/// A color written in a role: a palette name or a direct `#rrggbb` value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum ColorRef {
    Named(String),
    Hex(Rgb),
}

impl TryFrom<String> for ColorRef {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        if text.starts_with('#') {
            Rgb::parse_hex(&text).map(ColorRef::Hex)
        } else if text.is_empty() {
            Err("a color name must not be empty".to_string())
        } else {
            Ok(ColorRef::Named(text))
        }
    }
}

impl From<ColorRef> for String {
    fn from(color: ColorRef) -> String {
        match color {
            ColorRef::Named(name) => name,
            ColorRef::Hex(rgb) => rgb.to_hex(),
        }
    }
}

/// One role. A field left out (`None`) inherits from the `base` role when the
/// theme is resolved; this type only holds what the file wrote.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Role {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fg: Option<ColorRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<ColorRef>,
    /// Colors blended left to right across the text (banner title).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gradient: Option<Vec<ColorRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underline: Option<bool>,
}
