//! The frame a host paints: a cell buffer, a list of effects and a cursor request.
//!
//! The cell-buffer, rectangle, color and style types come from one external
//! crate and are re-exported here, so every other file uses them through this
//! module and a later swap touches this file only.

pub use ratatui_core::buffer::Buffer;
pub use ratatui_core::layout::Rect;
pub use ratatui_core::style::{Color, Modifier, Style};

use crate::effect::Effect;

/// Shape of the text cursor a host may draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorShape {
    #[default]
    Bar,
    Block,
    Underline,
}

/// Where the caret should be shown. The window host also uses the cell as the
/// caret rectangle it gives the input method for its candidate window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorRequest {
    pub col: u16,
    pub row: u16,
    pub shape: CursorShape,
}

/// Everything a host needs to show one picture of the screen.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    /// One styled symbol per grid cell.
    pub cells: Buffer,
    /// Drawn after the cells they cover; a host may ignore any of them.
    pub effects: Vec<Effect>,
    /// `None` hides the cursor.
    pub cursor: Option<CursorRequest>,
    /// How thick a host that draws frame lines itself makes them, in pixels
    /// at scale factor 1 (the host multiplies by its own scale). A terminal
    /// ignores it. At least 1.
    pub line_weight: u8,
}

impl Frame {
    /// An empty frame covering `area`, with no effects and no cursor, and
    /// the thinnest lines.
    pub fn new(area: Rect) -> Self {
        Self {
            cells: Buffer::empty(area),
            effects: Vec::new(),
            cursor: None,
            line_weight: 1,
        }
    }

    /// The grid area this frame covers.
    pub fn area(&self) -> Rect {
        self.cells.area
    }

    pub fn push_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }
}
