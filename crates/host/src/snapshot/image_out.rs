//! The canvas written out as an image.
//!
//! This is the same painting the window shows, so a picture can be taken on a
//! machine with no display at all: a build check, a golden picture, or a look
//! at the screen over a remote shell.

use std::fmt;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use crate::paint::Canvas;

#[derive(Debug)]
pub enum SnapshotError {
    Create(PathBuf, std::io::Error),
    Write(PathBuf, String),
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Create(path, error) => {
                write!(f, "the picture file {} could not be made: {error}", path.display())
            }
            Self::Write(path, reason) => {
                write!(f, "the picture {} was not written: {reason}", path.display())
            }
        }
    }
}

impl std::error::Error for SnapshotError {}

/// Writes the canvas to `path` as an image, three bytes per pixel.
pub fn write_png(canvas: &Canvas, path: &Path) -> Result<(), SnapshotError> {
    let file =
        File::create(path).map_err(|error| SnapshotError::Create(path.to_path_buf(), error))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), canvas.width(), canvas.height());
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let fail = |error: png::EncodingError| SnapshotError::Write(path.to_path_buf(), error.to_string());
    let mut writer = encoder.write_header().map_err(fail)?;
    writer.write_image_data(&canvas.rgb_bytes()).map_err(fail)?;
    writer.finish().map_err(fail)
}

#[cfg(test)]
include!("image_out.tests.rs");
