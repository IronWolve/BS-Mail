//! Changing one value in `config.toml` and nothing else. The rest of the
//! file (comments, key order, other tables, blank lines) stays as the user
//! wrote it. This file only declares and re-exports the modules.

mod atomic;
mod document;
mod problem;
mod save;
mod setting;

pub use save::save;
pub use setting::Setting;
