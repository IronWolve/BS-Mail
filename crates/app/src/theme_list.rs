//! The names the Theme choice on the Options screen goes through.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use ui::theme::{DEFAULT_THEME_NAME, builtin_theme_names};

use crate::config_io::{THEMES_DIR, is_plain_name};

/// Every built-in theme, and every `<name>.toml` in the data folder's themes
/// folder whose name is plain: sorted, each once, the default first. A name
/// with a space or a dot in it is not listed, because no setting could name it.
pub fn theme_names(dir: Option<&Path>) -> Vec<String> {
    let mut found: BTreeSet<String> = builtin_theme_names().iter().map(|name| (*name).to_string()).collect();
    if let Some(dir) = dir
        && let Ok(entries) = fs::read_dir(dir.join(THEMES_DIR))
    {
        for path in entries.flatten().map(|entry| entry.path()) {
            let stem = path.file_stem().and_then(OsStr::to_str);
            if path.extension() == Some(OsStr::new("toml"))
                && path.is_file()
                && let Some(stem) = stem.filter(|stem| is_plain_name(stem))
            {
                found.insert(stem.to_string());
            }
        }
    }
    let mut names = vec![DEFAULT_THEME_NAME.to_string()];
    names.extend(found.into_iter().filter(|name| name != DEFAULT_THEME_NAME));
    names
}

#[cfg(test)]
include!("theme_list.tests.rs");
