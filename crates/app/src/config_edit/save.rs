//! Saving one setting into the data folder's `config.toml`.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use super::atomic;
use super::document::with_setting;
use super::problem::SaveProblem;
use super::setting::Setting;
use crate::config_io::CONFIG_NAME;

/// Writes `setting` into `dir`'s `config.toml`, making the folder and the
/// file when they are not there. A file that is not valid TOML is not
/// touched.
pub fn save(dir: Option<&Path>, setting: &Setting) -> Result<(), SaveProblem> {
    let dir = dir.ok_or(SaveProblem::NoFolder)?;
    let path = dir.join(CONFIG_NAME);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => return Err(SaveProblem::Io(error)),
    };
    let changed = with_setting(&text, setting)?;
    if changed == text {
        return Ok(());
    }
    atomic::write(&path, &changed)?;
    Ok(())
}

#[cfg(test)]
include!("save.tests.rs");
