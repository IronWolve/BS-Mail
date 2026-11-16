//! Rebinding keys: the `[keys]` table of `config.toml` read into the keys in
//! force. One file per job: a key's spelling and matching (`spec`), the keys
//! no action may take (`rules`), the one list of action names (`actions`), and
//! the map that puts the table over the defaults and reports what it cannot
//! keep (`map`). This file only declares and re-exports the modules.

pub mod actions;
pub mod map;
pub mod rules;
pub mod spec;

pub use actions::{NOT_REBINDABLE, REBINDABLE, Rebindable};
pub use map::KeyMap;
pub use rules::FIXED_KEYS;
pub use spec::KeySpec;
