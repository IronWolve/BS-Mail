//! One cell style as a single SGR escape sequence. Every sequence starts from
//! a reset, so a terminal never inherits the colors of the cell before it.

use surface::{Color, Modifier, Style};

/// Turns every attribute off.
pub const RESET: &str = "\x1b[0m";

/// Base code of the eight standard colors: 30 for text, 40 for background.
const FG_BASE: u8 = 30;
const BG_BASE: u8 = 40;
/// Added to the base for the bright half of the sixteen.
const BRIGHT: u8 = 60;

/// The style as one sequence: `ESC [ 0 ; … m`.
pub fn sgr(style: &Style) -> String {
    let mut codes = vec!["0".to_string()];
    codes.extend(modifier_codes(style.add_modifier));
    if let Some(color) = style.fg {
        codes.push(color_code(color, FG_BASE));
    }
    if let Some(color) = style.bg {
        codes.push(color_code(color, BG_BASE));
    }
    format!("\x1b[{}m", codes.join(";"))
}

fn modifier_codes(modifier: Modifier) -> Vec<String> {
    let table = [
        (Modifier::BOLD, "1"),
        (Modifier::DIM, "2"),
        (Modifier::ITALIC, "3"),
        (Modifier::UNDERLINED, "4"),
        (Modifier::SLOW_BLINK, "5"),
        (Modifier::RAPID_BLINK, "6"),
        (Modifier::REVERSED, "7"),
        (Modifier::HIDDEN, "8"),
        (Modifier::CROSSED_OUT, "9"),
    ];
    table
        .iter()
        .filter(|(flag, _)| modifier.contains(*flag))
        .map(|(_, code)| (*code).to_string())
        .collect()
}

/// One color as its parameters. `base` is 30 for text and 40 for background,
/// so the same table serves both.
fn color_code(color: Color, base: u8) -> String {
    let standard = |offset: u8| (base + offset).to_string();
    match color {
        Color::Reset => standard(9),
        Color::Black => standard(0),
        Color::Red => standard(1),
        Color::Green => standard(2),
        Color::Yellow => standard(3),
        Color::Blue => standard(4),
        Color::Magenta => standard(5),
        Color::Cyan => standard(6),
        Color::Gray => standard(7),
        Color::DarkGray => standard(BRIGHT),
        Color::LightRed => standard(BRIGHT + 1),
        Color::LightGreen => standard(BRIGHT + 2),
        Color::LightYellow => standard(BRIGHT + 3),
        Color::LightBlue => standard(BRIGHT + 4),
        Color::LightMagenta => standard(BRIGHT + 5),
        Color::LightCyan => standard(BRIGHT + 6),
        Color::White => standard(BRIGHT + 7),
        Color::Rgb(r, g, b) => format!("{};2;{r};{g};{b}", base + 8),
        Color::Indexed(index) => format!("{};5;{index}", base + 8),
    }
}

#[cfg(test)]
include!("sgr.tests.rs");
