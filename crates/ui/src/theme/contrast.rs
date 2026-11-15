//! The WCAG 2 contrast ratio between two colors, and the check of every
//! built-in theme against it. Only the tests use it.

use model::theme_schema::Rgb;

/// The relative luminance of a color, 0 (black) to 1 (white).
pub fn luminance(color: Rgb) -> f64 {
    let channel = |value: u8| {
        let v = f64::from(value) / 255.0;
        if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b)
}

/// The ratio of the lighter color's luminance to the darker one's, each with
/// 0.05 added: 1 for equal colors, 21 for black and white.
pub fn contrast_ratio(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (light, dark) = if la >= lb { (la, lb) } else { (lb, la) };
    (light + 0.05) / (dark + 0.05)
}

#[cfg(test)]
include!("contrast.tests.rs");
