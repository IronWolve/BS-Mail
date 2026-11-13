//! An icon set: for each `IconId`, the text to draw in the fixed two-cell
//! icon slot and its cell width.

use model::config::IconMode;
use unicode_width::UnicodeWidthStr;

use super::icon_id::IconId;
use super::{ascii_set, nerd_set, unicode_set};

/// Every icon is drawn in a slot this many cells wide.
pub const SLOT_CELLS: u16 = 2;

/// Which glyph table an icon set draws from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconKind {
    Nerd,
    Unicode,
    Ascii,
}

/// What to draw for one icon: text that is exactly `cells` wide, padded with
/// spaces after a narrow glyph, or only spaces for an icon without a form in
/// this set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconSlot {
    pub text: String,
    pub cells: u16,
}

impl IconSlot {
    /// True when the set has no glyph for this icon and the slot is empty.
    pub fn is_blank(&self) -> bool {
        self.text.trim().is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct IconSet {
    kind: IconKind,
    slots: Vec<IconSlot>,
}

impl IconSet {
    pub fn new(kind: IconKind) -> Self {
        let slots = IconId::ALL
            .iter()
            .map(|id| fit_to_slot(raw_glyph(kind, *id)))
            .collect();
        Self { kind, slots }
    }

    /// Picks the set for the `[appearance] icons` option. `auto` is the kind
    /// the host decided on (the icon-font set in the window; in a terminal,
    /// the answer of the first-run icon check).
    pub fn from_mode(mode: IconMode, auto: IconKind) -> Self {
        let kind = match mode {
            IconMode::Auto => auto,
            IconMode::Nerd => IconKind::Nerd,
            IconMode::Unicode => IconKind::Unicode,
            IconMode::Ascii => IconKind::Ascii,
        };
        Self::new(kind)
    }

    pub fn kind(&self) -> IconKind {
        self.kind
    }

    pub fn slot(&self, id: IconId) -> &IconSlot {
        &self.slots[id.index()]
    }

    /// The text to draw for `id`; always `SLOT_CELLS` cells wide.
    pub fn text(&self, id: IconId) -> &str {
        &self.slot(id).text
    }
}

fn raw_glyph(kind: IconKind, id: IconId) -> &'static str {
    match kind {
        IconKind::Nerd => nerd_set::glyph(id),
        IconKind::Unicode => unicode_set::glyph(id),
        IconKind::Ascii => ascii_set::glyph(id),
    }
}

/// Fits a glyph to the slot: a one-cell glyph gets a trailing space, a
/// two-cell glyph stays as it is, and an empty or too wide one becomes a
/// blank slot so text never overlaps.
fn fit_to_slot(glyph: &str) -> IconSlot {
    let slot_width = usize::from(SLOT_CELLS);
    let width = UnicodeWidthStr::width(glyph);
    let text = match width {
        1 => format!("{glyph} "),
        2 => glyph.to_string(),
        _ => " ".repeat(slot_width),
    };
    IconSlot {
        text,
        cells: SLOT_CELLS,
    }
}

#[cfg(test)]
include!("icon_set.tests.rs");
