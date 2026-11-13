//! Which face actually has a character.
//!
//! The style's own face is tried first, then the regular face, which is the
//! patched one carrying the icons, then the emoji face. A character no face
//! has gets no glyph, and the painter draws a hollow box instead, so the
//! column never shifts.

use super::embedded::{FaceKind, FontSet};

/// The private-use range, where the icon glyphs live.
pub const PRIVATE_USE: std::ops::RangeInclusive<char> = '\u{e000}'..='\u{f8ff}';

/// True for a character that is a picture rather than a letter, which is
/// drawn to fill its slot instead of sitting on the baseline.
pub fn is_icon(ch: char) -> bool {
    PRIVATE_USE.contains(&ch)
}

/// A face and the glyph of that face to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    pub face: FaceKind,
    pub id: u16,
}

/// The face that has `ch`, starting from the one the style asked for.
pub fn choose(fonts: &FontSet, want: FaceKind, ch: char) -> Option<Glyph> {
    let order = [want, FaceKind::Regular, FaceKind::Emoji];
    let mut tried = [FaceKind::Regular; 3];
    let mut count = 0;
    for face in order {
        if tried[..count].contains(&face) {
            continue;
        }
        tried[count] = face;
        count += 1;
        if let Some(id) = glyph_in(fonts, face, ch) {
            return Some(Glyph { face, id });
        }
    }
    None
}

/// The glyph of one face, or `None` when that face has no glyph for `ch`.
fn glyph_in(fonts: &FontSet, face: FaceKind, ch: char) -> Option<u16> {
    let font = fonts.face(face)?;
    match font.charmap().map(ch) {
        0 => None,
        id => Some(id),
    }
}

#[cfg(test)]
include!("fallback.tests.rs");
