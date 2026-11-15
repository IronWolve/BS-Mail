//! Writing an account file the plan settled: whole or not at all, readable by
//! its owner only, with an old file kept as a backup beside it.

use std::fs;
use std::path::PathBuf;

use crate::account_writer::save_account;
use crate::private_dirs;

use super::plan::Plan;

/// What saving did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Saved {
    pub file: PathBuf,
    /// Where the account file that was there went, if there was one.
    pub kept_old_as: Option<PathBuf>,
}

/// Saves the account of `plan`. The folder is made first. Changing an account
/// keeps the keys of the old file that the form cannot express; adding one
/// refuses to write over a file that appeared since the plan was made.
pub fn save(plan: &Plan) -> Result<Saved, String> {
    let text = if plan.edit { unchanged_since_planned(plan)? } else { refuse_if_taken(plan)?; plan.text.clone() };
    if let Some(folder) = plan.file.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        private_dirs::make(folder).map_err(|error| format!("could not make the folder {}: {error}", folder.display()))?;
    }
    let saved = save_account(&plan.file, &text)?;
    Ok(Saved { file: plan.file.clone(), kept_old_as: saved.kept_old_as })
}

/// The text of a changed account. The plan already carries the old file's
/// keys, since that is the account that was tried; so the old file is read
/// once more and must still be what it was when the plan was made, or it was
/// changed by something else in the meantime and what was tried is no longer
/// what would be saved over it.
fn unchanged_since_planned(plan: &Plan) -> Result<String, String> {
    if fs::read_to_string(&plan.file).ok() != plan.old_text {
        return Err("the account file was changed while this was being tried; try again".to_string());
    }
    Ok(plan.text.clone())
}

/// A new account in a folder under the root must find no account file there.
/// For `--account <file>` the file is the account's own and is replaced, the
/// old one kept as a backup.
fn refuse_if_taken(plan: &Plan) -> Result<(), String> {
    if plan.exclusive && plan.file.exists() {
        return Err("an account was set up in that folder in the meantime; try again".to_string());
    }
    Ok(())
}

#[cfg(test)]
include!("save.tests.rs");
