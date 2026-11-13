//! What the binary tells the window before it opens.
//!
//! The host reads no config file and no theme: everything it needs about the
//! look arrives here, so the folder a run started from never reaches it.

use crate::paint::Palette;

#[derive(Debug, Clone, PartialEq)]
pub struct WindowOptions {
    /// The window's title bar.
    pub title: String,
    /// The grid the window is sized for at first start.
    pub cols: u16,
    pub rows: u16,
    /// The grid below which the window will not shrink.
    pub min_cols: u16,
    pub min_rows: u16,
    /// Text size in points, before the display's scale factor.
    pub points: u8,
    /// Multiplies the row height.
    pub line_spacing: f32,
    /// The colors a cell means by "reset".
    pub palette: Palette,
}

/// Points, the smallest and largest the font-size keys go.
pub const POINTS_MIN: u8 = 6;
pub const POINTS_MAX: u8 = 40;

impl WindowOptions {
    /// The size one step of the font-size keys lands on.
    pub fn stepped(points: u8, step: i8) -> u8 {
        let next = i16::from(points) + i16::from(step);
        next.clamp(i16::from(POINTS_MIN), i16::from(POINTS_MAX)) as u8
    }
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: String::new(),
            cols: 145,
            rows: 48,
            min_cols: 40,
            min_rows: 16,
            points: 12,
            line_spacing: 1.0,
            palette: Palette::default(),
        }
    }
}

#[cfg(test)]
include!("options.tests.rs");
