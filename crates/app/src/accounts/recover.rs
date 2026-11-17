//! Putting the accounts folder right after a run that ended in the middle of
//! adding or changing an account (killed, crashed, the terminal hung up).
//!
//! A clean exit gives the try up itself, so this only meets what a hard stop
//! leaves behind, and only runs once the data folder is locked, so no other
//! copy of the program can be in the middle of such a try:
//!
//! - A sign-in whose tokens replaced an account's earlier ones before the
//!   account was saved (`engine::previous_token`, an attempt marker in the
//!   vault): the earlier tokens come back, or, for an account that was never
//!   saved, the new tokens go. A run of an older version that stopped there
//!   left `oauth/<id>.token.prev` instead (`engine::legacy_attempt`), which is
//!   put back the same way, and the move into the vault then takes the file in.
//! - `account.toml.new` and `account.toml.bak.new`, the temporary names a save
//!   writes under before the rename: they hold account text (the password is
//!   in the vault, but the text may be an older file's), and nothing lists them.
//!
//! Nothing else is removed. A folder with a saved sign-in and no account file
//! that no try of this program left a trace of is only reported. The vault's
//! own entries for an account that has no folder are not looked for.

use std::fs;
use std::path::Path;

use engine::{TokenPlace, legacy_attempt, previous_token, token_file_in, token_store};

use crate::paths::ACCOUNT_FILE;

use super::id::is_valid_id;

/// Fixes what an interrupted try left in the account folders under `root`;
/// the sign-in tokens are in the vault in `secrets`. Returns one line for each
/// problem it could not fix or only found.
pub fn recover(root: &Path, secrets: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut folders: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| is_valid_id(name))
        .collect();
    folders.sort();
    folders.iter().filter_map(|id| tidy(root, secrets, id).err()).collect()
}

/// One account folder.
fn tidy(root: &Path, secrets: &Path, id: &str) -> Result<(), String> {
    let folder = root.join(id);
    let token = token_file_in(&folder, id);
    let place = TokenPlace::new(secrets, id, &token);
    let interrupted = legacy_attempt::is_pending(&token) || previous_token::is_pending(&place);
    legacy_attempt::restore(&token).map_err(|why| format!("account {id}: {why}"))?;
    previous_token::restore(&place).map_err(|why| format!("account {id}: {why}"))?;
    let leftovers = [format!("{ACCOUNT_FILE}.new"), format!("{ACCOUNT_FILE}.bak.new")];
    for name in leftovers {
        let path = folder.join(&name);
        if fs::symlink_metadata(&path).is_ok_and(|meta| meta.is_file()) {
            fs::remove_file(&path).map_err(|why| format!("account {id}: could not remove {name}: {why}"))?;
        }
    }
    if folder.join(ACCOUNT_FILE).exists() {
        return Ok(());
    }
    if interrupted {
        // A new account that never got saved: only empty folders go.
        let _ = fs::remove_dir(token.parent().unwrap_or(&folder));
        let _ = fs::remove_dir(&folder);
    } else if token.exists() || token_store::read(&place).is_ok() {
        return Err(format!(
            "the folder \"{id}\" in the accounts folder holds a saved sign-in but no account file; \
             delete the folder if you do not need it"
        ));
    }
    Ok(())
}

#[cfg(test)]
include!("recover.tests.rs");
