//! The eleven pieces of a border style: four corners, two lines, four tees
//! and the cross, so shared borders join correctly in every style.

use model::config::BorderStyle;

/// Box-drawing pieces of one style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorderSet {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
    /// Horizontal line with a branch going down (`┬`).
    pub tee_down: char,
    /// Horizontal line with a branch going up (`┴`).
    pub tee_up: char,
    /// Vertical line with a branch going right (`├`).
    pub tee_right: char,
    /// Vertical line with a branch going left (`┤`).
    pub tee_left: char,
    pub cross: char,
}

/// Rounded corners. Unicode has no rounded tees or cross, so those are the
/// plain pieces; the window host draws all of them as shapes anyway.
pub const ROUNDED: BorderSet = BorderSet {
    top_left: '\u{256d}',
    top_right: '\u{256e}',
    bottom_left: '\u{2570}',
    bottom_right: '\u{256f}',
    horizontal: '\u{2500}',
    vertical: '\u{2502}',
    tee_down: '\u{252c}',
    tee_up: '\u{2534}',
    tee_right: '\u{251c}',
    tee_left: '\u{2524}',
    cross: '\u{253c}',
};

pub const PLAIN: BorderSet = BorderSet {
    top_left: '\u{250c}',
    top_right: '\u{2510}',
    bottom_left: '\u{2514}',
    bottom_right: '\u{2518}',
    ..ROUNDED
};

pub const THICK: BorderSet = BorderSet {
    top_left: '\u{250f}',
    top_right: '\u{2513}',
    bottom_left: '\u{2517}',
    bottom_right: '\u{251b}',
    horizontal: '\u{2501}',
    vertical: '\u{2503}',
    tee_down: '\u{2533}',
    tee_up: '\u{253b}',
    tee_right: '\u{2523}',
    tee_left: '\u{252b}',
    cross: '\u{254b}',
};

pub const DOUBLE: BorderSet = BorderSet {
    top_left: '\u{2554}',
    top_right: '\u{2557}',
    bottom_left: '\u{255a}',
    bottom_right: '\u{255d}',
    horizontal: '\u{2550}',
    vertical: '\u{2551}',
    tee_down: '\u{2566}',
    tee_up: '\u{2569}',
    tee_right: '\u{2560}',
    tee_left: '\u{2563}',
    cross: '\u{256c}',
};

impl BorderSet {
    /// The set for the `[appearance] border` option.
    pub fn for_style(style: BorderStyle) -> &'static BorderSet {
        match style {
            BorderStyle::Rounded => &ROUNDED,
            BorderStyle::Plain => &PLAIN,
            BorderStyle::Thick => &THICK,
            BorderStyle::Double => &DOUBLE,
        }
    }

    /// All eleven pieces, corners first.
    pub fn pieces(&self) -> [char; 11] {
        [
            self.top_left,
            self.top_right,
            self.bottom_left,
            self.bottom_right,
            self.horizontal,
            self.vertical,
            self.tee_down,
            self.tee_up,
            self.tee_right,
            self.tee_left,
            self.cross,
        ]
    }
}

#[cfg(test)]
include!("border_set.tests.rs");
