//! What the terminal can show, worked out from its environment.

use surface::ColorDepth;

/// The color depth of the terminal. `NO_COLOR` set to anything turns color
/// off; `COLORTERM` says true color outright; otherwise `TERM` is read for
/// 256 colors, and a terminal that says nothing is taken to have sixteen.
pub fn color_depth(env: &dyn Fn(&str) -> Option<String>) -> ColorDepth {
    if env("NO_COLOR").is_some_and(|value| !value.is_empty()) {
        return ColorDepth::None;
    }
    if env("COLORTERM").is_some_and(|value| matches!(value.as_str(), "truecolor" | "24bit")) {
        return ColorDepth::TrueColor;
    }
    match env("TERM").as_deref() {
        Some("dumb") | Some("") => ColorDepth::None,
        Some(term) if term.contains("256color") || term.contains("direct") => ColorDepth::Indexed256,
        _ => ColorDepth::Ansi16,
    }
}

/// `color_depth` for the real environment.
pub fn color_depth_from_env() -> ColorDepth {
    color_depth(&|name| std::env::var(name).ok())
}

/// A depth named by the user: `truecolor`, `256`, `16` or `none`.
pub fn parse_depth(name: &str) -> Option<ColorDepth> {
    match name {
        "truecolor" | "24bit" => Some(ColorDepth::TrueColor),
        "256" => Some(ColorDepth::Indexed256),
        "16" => Some(ColorDepth::Ansi16),
        "none" | "0" => Some(ColorDepth::None),
        _ => None,
    }
}

#[cfg(test)]
include!("probe.tests.rs");
