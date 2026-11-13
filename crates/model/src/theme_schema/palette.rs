//! Palette types: hex colors, a named palette and the sixteen terminal color names.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// An opaque color. In theme files it is written as `#rrggbb`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    /// Parses `#rrggbb`; the error text names the offending value.
    pub fn parse_hex(text: &str) -> Result<Self, String> {
        let digits = text
            .strip_prefix('#')
            .ok_or_else(|| format!("color `{text}` must start with `#`"))?;
        if digits.len() != 6 || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("color `{text}` must be written as `#rrggbb`"));
        }
        let channel = |at: usize| {
            u8::from_str_radix(&digits[at..at + 2], 16)
                .map_err(|err| format!("color `{text}`: {err}"))
        };
        Ok(Self {
            r: channel(0)?,
            g: channel(2)?,
            b: channel(4)?,
        })
    }

    /// The `#rrggbb` form, lower case.
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl TryFrom<String> for Rgb {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        Self::parse_hex(&text)
    }
}

impl From<Rgb> for String {
    fn from(color: Rgb) -> String {
        color.to_hex()
    }
}

/// Named colors of one theme: `[palette]` in the file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Palette {
    pub colors: BTreeMap<String, Rgb>,
}

impl Palette {
    pub fn get(&self, name: &str) -> Option<Rgb> {
        self.colors.get(name).copied()
    }
}

/// The sixteen standard terminal colors, written in kebab case in `[ansi16]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Ansi16Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}
