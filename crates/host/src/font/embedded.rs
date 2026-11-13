//! The faces the binary hands to the host: four text faces and an optional
//! emoji face. Nothing here knows where the files came from, so a user's own
//! font file arrives the same way as the built-in one.

use std::fmt;

use swash::FontRef;

/// Which face a cell's style asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaceKind {
    Regular,
    Bold,
    Italic,
    BoldItalic,
    Emoji,
}

impl FaceKind {
    /// The text face for a style's weight and slant.
    pub fn text(bold: bool, italic: bool) -> Self {
        match (bold, italic) {
            (false, false) => Self::Regular,
            (true, false) => Self::Bold,
            (false, true) => Self::Italic,
            (true, true) => Self::BoldItalic,
        }
    }

    /// Index into the text faces; the emoji face is held apart.
    fn index(self) -> Option<usize> {
        match self {
            Self::Regular => Some(0),
            Self::Bold => Some(1),
            Self::Italic => Some(2),
            Self::BoldItalic => Some(3),
            Self::Emoji => None,
        }
    }
}

/// A face that did not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontError(pub String);

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for FontError {}

/// The faces one run draws with.
pub struct FontSet {
    text: [Vec<u8>; 4],
    emoji: Option<Vec<u8>>,
}

/// Sizes, never the bytes: a face is megabytes long.
impl fmt::Debug for FontSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text: Vec<usize> = self.text.iter().map(Vec::len).collect();
        f.debug_struct("FontSet")
            .field("text_bytes", &text)
            .field("emoji_bytes", &self.emoji.as_ref().map(Vec::len))
            .finish()
    }
}

impl FontSet {
    /// Checks every face and keeps the bytes. The regular face is the one the
    /// cell size comes from, so a set without it is an error.
    pub fn new(
        text: [Vec<u8>; 4],
        emoji: Option<Vec<u8>>,
    ) -> Result<Self, FontError> {
        let names = ["regular", "bold", "italic", "bold italic"];
        for (data, name) in text.iter().zip(names) {
            if FontRef::from_index(data, 0).is_none() {
                return Err(FontError(format!("the {name} face did not parse")));
            }
        }
        if let Some(data) = &emoji
            && FontRef::from_index(data, 0).is_none()
        {
            return Err(FontError("the emoji face did not parse".to_string()));
        }
        Ok(Self { text, emoji })
    }

    /// One face, or `None` when this set has no emoji face.
    pub fn face(&self, kind: FaceKind) -> Option<FontRef<'_>> {
        let data = match kind.index() {
            Some(index) => &self.text[index],
            None => self.emoji.as_ref()?,
        };
        FontRef::from_index(data, 0)
    }

    /// The face the cell size is measured from. Checked when the set was
    /// built, so it is always there.
    pub fn regular(&self) -> FontRef<'_> {
        self.face(FaceKind::Regular)
            .expect("the regular face parsed when the set was built")
    }

    pub fn has_emoji(&self) -> bool {
        self.emoji.is_some()
    }
}

#[cfg(test)]
include!("embedded.tests.rs");
