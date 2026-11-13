//! The faces compiled into the program.
//!
//! The folder they come from is decided by the build script, so no path and
//! no folder name appears here. Every build carries the same faces, which is
//! why the window looks the same on every machine.

use host::{FontError, FontSet};

const REGULAR: &[u8] = include_bytes!(concat!(env!("APP_FONT_DIR"), "/text-regular.ttf"));
const BOLD: &[u8] = include_bytes!(concat!(env!("APP_FONT_DIR"), "/text-bold.ttf"));
const ITALIC: &[u8] = include_bytes!(concat!(env!("APP_FONT_DIR"), "/text-italic.ttf"));
const BOLD_ITALIC: &[u8] = include_bytes!(concat!(env!("APP_FONT_DIR"), "/text-bold-italic.ttf"));
const EMOJI: &[u8] = include_bytes!(concat!(env!("APP_FONT_DIR"), "/emoji.ttf"));

/// The set the window draws with.
pub fn embedded() -> Result<FontSet, FontError> {
    FontSet::new(
        [
            REGULAR.to_vec(),
            BOLD.to_vec(),
            ITALIC.to_vec(),
            BOLD_ITALIC.to_vec(),
        ],
        Some(EMOJI.to_vec()),
    )
}

/// How many bytes of faces the program carries.
pub fn embedded_bytes() -> usize {
    [REGULAR, BOLD, ITALIC, BOLD_ITALIC, EMOJI]
        .iter()
        .map(|face| face.len())
        .sum()
}

#[cfg(test)]
include!("fonts.tests.rs");
