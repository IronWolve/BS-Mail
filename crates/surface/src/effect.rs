//! Effects: things a window host draws on top of the cell grid and a terminal
//! host ignores. The cells under an effect still read correctly without it.

use crate::frame::{Color, Rect};

/// How the glyphs of a scaled text are colored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextFill {
    Solid(Color),
    /// Colors blended left to right across the text.
    Gradient(Vec<Color>),
}

/// A soft glow around scaled text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glow {
    pub color: Color,
    /// Radius in pixels at scale 1.
    pub radius: u8,
}

/// Text at an integer multiple of the cell size (the banner title).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaledText {
    /// Cells the text covers: its width in cells and `scale` rows high.
    pub area: Rect,
    pub text: String,
    /// 2 means double width and double height.
    pub scale: u8,
    pub fill: TextFill,
    pub glow: Option<Glow>,
    pub bold: bool,
    pub italic: bool,
}

/// A run of cells drawn half a row lower, to center normal-size text beside
/// double-size text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HalfRowShift {
    pub area: Rect,
}

/// A rounded background behind a run of cells (keycaps).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoundedFill {
    pub area: Rect,
    pub color: Color,
    /// Corner radius in cell widths.
    pub radius: f32,
}

/// A rounded outline around a one-row bar (the storage gauge).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GaugeOutline {
    pub area: Rect,
    pub color: Color,
    /// Corner radius in cell widths.
    pub radius: f32,
    /// Line thickness in pixels at scale 1.
    pub weight: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    ScaledText(ScaledText),
    HalfRowShift(HalfRowShift),
    RoundedFill(RoundedFill),
    GaugeOutline(GaugeOutline),
}

impl Effect {
    /// The cells this effect covers.
    pub fn area(&self) -> Rect {
        match self {
            Effect::ScaledText(effect) => effect.area,
            Effect::HalfRowShift(effect) => effect.area,
            Effect::RoundedFill(effect) => effect.area,
            Effect::GaugeOutline(effect) => effect.area,
        }
    }
}
