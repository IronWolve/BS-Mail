//! Cell colors as pixels.
//!
//! The theme resolves to true color, so most cells name their own red, green
//! and blue. The named and numbered colors are here for a theme or a user
//! file that uses them, and `Reset` means the screen's own base colors, which
//! the binary passes in because the host does not read themes.

use surface::Color;

/// One opaque color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Red, green and blue in the low three bytes, as a window buffer wants it.
    pub const fn packed(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | self.b as u32
    }

    pub const fn unpack(value: u32) -> Self {
        Self::new((value >> 16) as u8, (value >> 8) as u8, value as u8)
    }

    /// `self` at `amount` of 255, over `other`.
    pub fn mix(self, other: Self, amount: u8) -> Self {
        let blend = |a: u8, b: u8| {
            let a = u32::from(a) * u32::from(amount);
            let b = u32::from(b) * u32::from(255 - amount);
            ((a + b + 127) / 255) as u8
        };
        Self::new(blend(self.r, other.r), blend(self.g, other.g), blend(self.b, other.b))
    }
}

/// Whether a color is being asked for as text or as background.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    Foreground,
    Background,
}

/// What `Color::Reset` means on this screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub fg: Rgb,
    pub bg: Rgb,
}

impl Palette {
    pub const fn new(fg: Rgb, bg: Rgb) -> Self {
        Self { fg, bg }
    }

    /// The pixels for one cell color.
    pub fn resolve(&self, color: Color, ink: Ink) -> Rgb {
        match color {
            Color::Reset => match ink {
                Ink::Foreground => self.fg,
                Ink::Background => self.bg,
            },
            Color::Rgb(r, g, b) => Rgb::new(r, g, b),
            Color::Indexed(index) => indexed(index),
            named => indexed(standard_index(named)),
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::new(Rgb::new(0xe6, 0xed, 0xf3), Rgb::new(0x0a, 0x0e, 0x1a))
    }
}

/// The sixteen terminal colors, dark half then bright half.
const STANDARD: [Rgb; 16] = [
    Rgb::new(0x00, 0x00, 0x00),
    Rgb::new(0xcd, 0x00, 0x00),
    Rgb::new(0x00, 0xcd, 0x00),
    Rgb::new(0xcd, 0xcd, 0x00),
    Rgb::new(0x00, 0x00, 0xee),
    Rgb::new(0xcd, 0x00, 0xcd),
    Rgb::new(0x00, 0xcd, 0xcd),
    Rgb::new(0xe5, 0xe5, 0xe5),
    Rgb::new(0x7f, 0x7f, 0x7f),
    Rgb::new(0xff, 0x00, 0x00),
    Rgb::new(0x00, 0xff, 0x00),
    Rgb::new(0xff, 0xff, 0x00),
    Rgb::new(0x5c, 0x5c, 0xff),
    Rgb::new(0xff, 0x00, 0xff),
    Rgb::new(0x00, 0xff, 0xff),
    Rgb::new(0xff, 0xff, 0xff),
];

/// Where a named color sits in the sixteen.
fn standard_index(color: Color) -> u8 {
    match color {
        Color::Black => 0,
        Color::Red => 1,
        Color::Green => 2,
        Color::Yellow => 3,
        Color::Blue => 4,
        Color::Magenta => 5,
        Color::Cyan => 6,
        Color::Gray => 7,
        Color::DarkGray => 8,
        Color::LightRed => 9,
        Color::LightGreen => 10,
        Color::LightYellow => 11,
        Color::LightBlue => 12,
        Color::LightMagenta => 13,
        Color::LightCyan => 14,
        _ => 15,
    }
}

/// The 256-color table: sixteen named, a six-level cube, then the grays.
fn indexed(index: u8) -> Rgb {
    const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
    match index {
        0..=15 => STANDARD[index as usize],
        16..=231 => {
            let value = index - 16;
            Rgb::new(
                LEVELS[(value / 36) as usize],
                LEVELS[((value / 6) % 6) as usize],
                LEVELS[(value % 6) as usize],
            )
        }
        _ => {
            let gray = 8 + 10 * (index - 232);
            Rgb::new(gray, gray, gray)
        }
    }
}

#[cfg(test)]
include!("colors.tests.rs");
