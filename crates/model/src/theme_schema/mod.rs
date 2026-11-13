//! Serde types for theme files; this file only declares and re-exports them.

pub mod palette;
pub mod role;
pub mod theme_file;

pub use palette::{Ansi16Color, Palette, Rgb};
pub use role::{BASE_ROLE, ColorRef, Role};
pub use theme_file::ThemeFile;
