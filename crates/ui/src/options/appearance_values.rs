//! The values the Appearance page steps through: which theme or icon set
//! comes next, and the next text size or line spacing within their limits.

use model::config::IconMode;
use model::config::font::{FONT_SIZE_MAX, FONT_SIZE_MIN, LINE_SPACING_MAX, LINE_SPACING_MIN};

/// One step of the line spacing.
pub const SPACING_STEP: f32 = 0.05;

const ICON_MODES: [IconMode; 4] = [IconMode::Auto, IconMode::Nerd, IconMode::Unicode, IconMode::Ascii];

pub fn icon_name(mode: IconMode) -> &'static str {
    match mode {
        IconMode::Auto => "auto",
        IconMode::Nerd => "nerd",
        IconMode::Unicode => "unicode",
        IconMode::Ascii => "ascii",
    }
}

/// The next or previous icon set, going round.
pub fn cycle_icons(mode: IconMode, forward: bool) -> IconMode {
    let at = ICON_MODES.iter().position(|candidate| *candidate == mode).unwrap_or(0);
    let count = ICON_MODES.len();
    ICON_MODES[if forward { (at + 1) % count } else { (at + count - 1) % count }]
}

/// The theme after or before `current` in `names`, going round. `None` when
/// there is no other theme to go to. A current theme that is not in the list
/// goes to the first name forward and the last backward.
pub fn cycle_theme(names: &[String], current: &str, forward: bool) -> Option<String> {
    let count = names.len();
    let next = match (names.iter().position(|name| name == current), forward) {
        (Some(at), true) => (at + 1) % count,
        (Some(at), false) => (at + count - 1) % count,
        (None, true) if count > 0 => 0,
        (None, false) if count > 0 => count - 1,
        _ => return None,
    };
    (names[next] != current).then(|| names[next].clone())
}

/// One point up or down, within the limits.
pub fn step_font_size(size: u8, plus: bool) -> u8 {
    let next = if plus { size.saturating_add(1) } else { size.saturating_sub(1) };
    next.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX)
}

/// The next multiple of 0.05 up or down, within the limits, to two places.
pub fn step_line_spacing(value: f32, plus: bool) -> f32 {
    // A spacing that is no number (the file can say `nan`) steps from the
    // plain row height; every comparison with it would otherwise fail and the
    // buttons would stay stuck on it.
    let value = if value.is_nan() { 1.0 } else { value };
    let scaled = value / SPACING_STEP;
    let steps = if plus { (scaled + 1e-3).floor() + 1.0 } else { (scaled - 1e-3).ceil() - 1.0 };
    let next = (steps * SPACING_STEP).clamp(LINE_SPACING_MIN, LINE_SPACING_MAX);
    (next * 100.0).round() / 100.0
}

/// Whether two spacings differ by more than rounding. A spacing that is no
/// number differs from every other, or a press that leaves it would be lost.
pub fn spacing_differs(a: f32, b: f32) -> bool {
    a.is_nan() != b.is_nan() || (a - b).abs() > 1e-4
}

pub fn spacing_text(value: f32) -> String {
    format!("{value:.2}")
}

#[cfg(test)]
include!("appearance_values.tests.rs");
