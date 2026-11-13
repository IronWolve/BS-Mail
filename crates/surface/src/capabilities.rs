//! What a host can do, so the UI can pick a banner, a palette depth and a
//! set of controls that the host actually supports.

/// Which effects a host draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EffectSupport {
    pub scaled_text: bool,
    pub half_row_shift: bool,
    pub rounded_fill: bool,
    pub gauge_outline: bool,
}

impl EffectSupport {
    pub fn all() -> Self {
        Self {
            scaled_text: true,
            half_row_shift: true,
            rounded_fill: true,
            gauge_outline: true,
        }
    }

    pub fn none() -> Self {
        Self::default()
    }
}

/// How many colors the host shows; the UI maps theme colors to this depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorDepth {
    TrueColor,
    Indexed256,
    Ansi16,
    /// No color: weight and underline only.
    None,
}

/// How copy and paste reach the system clipboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClipboardKind {
    /// Direct access to the system clipboard.
    Native,
    /// Copy only, through a terminal escape sequence.
    Escape,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    pub effects: EffectSupport,
    pub color_depth: ColorDepth,
    /// Pointer motion without a button held is reported.
    pub hover: bool,
    /// The host can change the pointer shape.
    pub pointer_shapes: bool,
    pub clipboard: ClipboardKind,
    /// The host can change the font size at run time.
    pub font_zoom: bool,
}
