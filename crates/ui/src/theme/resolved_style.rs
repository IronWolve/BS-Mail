//! The concrete style a role resolves to, and the one place where theme colors
//! become surface colors.

use model::theme_schema::Rgb;
use surface::{Color, Modifier, Style};

/// A role with every inherited field filled in: concrete colors and weights.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedStyle {
    pub fg: Rgb,
    /// `None` when neither the role, its parents nor `base` set a background.
    pub bg: Option<Rgb>,
    /// Colors blended left to right across the text; `fg` is the first stop,
    /// for hosts and widgets that draw one color.
    pub gradient: Option<Vec<Rgb>>,
    pub bold: bool,
    pub italic: bool,
    pub dim: bool,
    pub underline: bool,
}

/// Turns a theme color into a surface color.
///
/// Truecolor only. Color-depth mapping (256 and 16 colors) will replace this
/// one function, so every caller goes through it.
pub fn surface_color(rgb: Rgb) -> Color {
    Color::Rgb(rgb.r, rgb.g, rgb.b)
}

impl ResolvedStyle {
    /// The weight and decoration flags as surface modifiers.
    pub fn modifiers(&self) -> Modifier {
        let mut modifier = Modifier::empty();
        if self.bold {
            modifier |= Modifier::BOLD;
        }
        if self.italic {
            modifier |= Modifier::ITALIC;
        }
        if self.dim {
            modifier |= Modifier::DIM;
        }
        if self.underline {
            modifier |= Modifier::UNDERLINED;
        }
        modifier
    }

    /// Foreground, background (when set) and modifiers.
    pub fn to_style(&self) -> Style {
        let style = self.to_fg_style();
        match self.bg {
            Some(bg) => style.bg(surface_color(bg)),
            None => style,
        }
    }

    /// Foreground and modifiers only, for text drawn over a row or bar that
    /// already has its own background (the selected message row).
    pub fn to_fg_style(&self) -> Style {
        Style::new()
            .fg(surface_color(self.fg))
            .add_modifier(self.modifiers())
    }

    /// Background alone, for filling a region.
    pub fn to_bg_style(&self) -> Style {
        match self.bg {
            Some(bg) => Style::new().bg(surface_color(bg)),
            None => Style::new(),
        }
    }
}

#[cfg(test)]
include!("resolved_style.tests.rs");
