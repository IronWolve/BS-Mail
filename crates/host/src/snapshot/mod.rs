//! A frame as an image file, with no window involved.

pub mod image_out;

pub use image_out::{SnapshotError, write_png};
