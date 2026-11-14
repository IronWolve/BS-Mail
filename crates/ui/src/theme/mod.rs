//! Theme system: the embedded default, the file loader, role resolution,
//! gradients and the role-name constants. This file only declares and
//! re-exports them.

pub mod builtin;
pub mod gradient;
pub mod loader;
pub mod resolve;
pub mod resolved_style;
pub mod roles;

pub use builtin::{DEFAULT_THEME_NAME, builtin_theme_names, builtin_theme_text, default_theme, default_theme_text};
pub use gradient::{color_at, spread};
pub use loader::{ThemeError, load_theme_file, merge_inherited, parse_theme_file, validate};
pub use resolve::Theme;
pub use resolved_style::{ResolvedStyle, surface_color};
