//! Theme colors for a terminal that shows fewer than sixteen million.
//!
//! The theme is true color. A terminal with 256 colors gets the nearest entry
//! of the 6x6x6 cube or the gray ramp; one with sixteen gets the nearest of
//! the sixteen. The first sixteen entries of the 256 are never picked, since
//! a terminal's own palette decides what they look like.

use surface::{Color, ColorDepth};

/// The six levels of one axis of the 256-color cube.
const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// The sixteen standard colors as most terminals draw them, in the order of
/// their SGR codes: eight dark, then eight bright.
const ANSI16: [(u8, u8, u8); 16] = [
    (0, 0, 0),
    (205, 0, 0),
    (0, 205, 0),
    (205, 205, 0),
    (0, 0, 238),
    (205, 0, 205),
    (0, 205, 205),
    (229, 229, 229),
    (127, 127, 127),
    (255, 0, 0),
    (0, 255, 0),
    (255, 255, 0),
    (92, 92, 255),
    (255, 0, 255),
    (0, 255, 255),
    (255, 255, 255),
];

/// `color` as the terminal can show it at `depth`.
pub fn map_color(color: Color, depth: ColorDepth) -> Color {
    match (color, depth) {
        (Color::Rgb(r, g, b), ColorDepth::Indexed256) => Color::Indexed(nearest_256(r, g, b)),
        (Color::Rgb(r, g, b), ColorDepth::Ansi16) => named(nearest_16(r, g, b)),
        (Color::Indexed(index), ColorDepth::Ansi16) => {
            let (r, g, b) = index_rgb(index);
            named(nearest_16(r, g, b))
        }
        (_, ColorDepth::None) => Color::Reset,
        (other, _) => other,
    }
}

/// The nearest cube or gray entry, whichever is closer.
pub fn nearest_256(r: u8, g: u8, b: u8) -> u8 {
    let level = |value: u8| {
        (0..6)
            .min_by_key(|index| i32::from(LEVELS[*index]).abs_diff(i32::from(value)))
            .unwrap_or(0)
    };
    let (ri, gi, bi) = (level(r), level(g), level(b));
    let cube = (LEVELS[ri], LEVELS[gi], LEVELS[bi]);
    let cube_index = 16 + 36 * ri + 6 * gi + bi;
    let gray_step = ((i32::from(r) + i32::from(g) + i32::from(b)) / 3 - 8).clamp(0, 230) / 10;
    let gray_level = (8 + gray_step * 10) as u8;
    let gray = (gray_level, gray_level, gray_level);
    if distance((r, g, b), gray) < distance((r, g, b), cube) {
        232 + gray_step as u8
    } else {
        cube_index as u8
    }
}

/// The index of the nearest of the sixteen.
pub fn nearest_16(r: u8, g: u8, b: u8) -> usize {
    (0..16)
        .min_by_key(|index| distance((r, g, b), ANSI16[*index]))
        .unwrap_or(0)
}

fn distance(a: (u8, u8, u8), b: (u8, u8, u8)) -> u32 {
    let d = |x: u8, y: u8| u32::from(x.abs_diff(y)).pow(2);
    d(a.0, b.0) + d(a.1, b.1) + d(a.2, b.2)
}

/// The color a 256-color index stands for.
fn index_rgb(index: u8) -> (u8, u8, u8) {
    match index {
        0..=15 => ANSI16[usize::from(index)],
        16..=231 => {
            let value = usize::from(index - 16);
            (LEVELS[value / 36], LEVELS[value / 6 % 6], LEVELS[value % 6])
        }
        _ => {
            let gray = 8 + 10 * (index - 232);
            (gray, gray, gray)
        }
    }
}

/// The named color for one of the sixteen.
fn named(index: usize) -> Color {
    const NAMES: [Color; 16] = [
        Color::Black,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::Gray,
        Color::DarkGray,
        Color::LightRed,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
        Color::White,
    ];
    NAMES[index]
}

#[cfg(test)]
include!("color_map.tests.rs");
