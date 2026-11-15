//! The screens that cover the base screen. Stage 1 has none open; the kinds
//! named here are the ones the plan lists first.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayKind {
    Help,
    About,
}

/// At most one overlay is open at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Overlays {
    pub open: Option<OverlayKind>,
    /// The first help row shown, when the grid is too short for all of them.
    pub help_scroll: usize,
}

impl Overlays {
    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }
}

#[cfg(test)]
include!("overlays.tests.rs");
