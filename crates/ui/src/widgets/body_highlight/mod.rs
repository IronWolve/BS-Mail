//! Content coloring for plain body text: bullets and lead words, search
//! tokens, versions, paths, links, signatures, quotes and the application
//! name. Each rule is switchable. This file only declares and re-exports the
//! modules.

pub mod bullets;
pub mod found;
pub mod line;
pub mod links;
pub mod names;
pub mod rules;
pub mod tokens;

pub use line::{highlight_line, highlight_text};
pub use rules::HighlightRules;
