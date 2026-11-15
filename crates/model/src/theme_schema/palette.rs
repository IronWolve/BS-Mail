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

impl Ansi16Color {
    /// The color most terminals draw for this name. A frame that carries it
    /// is mapped back to the same terminal color, which is how a 16-color
    /// terminal gets the color the theme chose and not just the nearest one.
    pub fn rgb(self) -> Rgb {
        let (r, g, b) = match self {
            Self::Black => (0, 0, 0),
            Self::Red => (205, 0, 0),
            Self::Green => (0, 205, 0),
            Self::Yellow => (205, 205, 0),
            Self::Blue => (0, 0, 238),
            Self::Magenta => (205, 0, 205),
            Self::Cyan => (0, 205, 205),
            Self::White => (229, 229, 229),
            Self::BrightBlack => (127, 127, 127),
            Self::BrightRed => (255, 0, 0),
            Self::BrightGreen => (0, 255, 0),
            Self::BrightYellow => (255, 255, 0),
            Self::BrightBlue => (92, 92, 255),
            Self::BrightMagenta => (255, 0, 255),
            Self::BrightCyan => (0, 255, 255),
            Self::BrightWhite => (255, 255, 255),
        };
        Rgb { r, g, b }
    }
}
