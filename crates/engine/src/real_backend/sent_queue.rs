//! The copies of sent mail kept on disk until the mailbox has them.
//!
//! The message was already sent, so its copy is the only record the user has
//! of what went out. Each copy is one `.eml` file in the account's own
//! `sent_copies` folder, named for the account, written whole before anything tries to save it on the
//! server and removed once the server has it. A copy left over from a run that
//! ended before it was saved is found again when the next run starts. An
//! account with no folder of its own keeps nothing on disk.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use super::save_sent::SentCopy;
use crate::account_config::AccountConfig;
use crate::private_fs::{make_folder, write_new};
use crate::send::now_unix;

/// The folder inside the account's folder.
pub(super) const FOLDER: &str = "sent_copies";
const EXTENSION: &str = "eml";

/// Where one account keeps its unsaved copies. The folder may be shared with
/// other accounts (a folder holding several account files), so a copy's file
/// name starts with the account's id and a dot, which an id cannot contain.
#[derive(Debug, Clone, Default)]
pub(super) struct SentQueue {
    dir: Option<PathBuf>,
    id: String,
}

impl SentQueue {
    pub(super) fn for_account(config: &AccountConfig) -> Self {
        Self { dir: config.state_dir.as_ref().map(|dir| dir.join(FOLDER)), id: config.id.clone() }
    }

    /// Whether copies are meant to be kept on disk: the account has a folder
    /// of its own. A copy that then cannot be written is a failure to report;
    /// an account with no such folder never meant to keep files.
    pub(super) fn keeps_files(&self) -> bool {
        self.dir.is_some()
    }

    /// Keeps `bytes` and says where, or `None` when they cannot be kept on
    /// disk (no folder, no room, no permission): the copy then lives only in
    /// memory, which is what it did before.
    pub(super) fn store(&self, bytes: &[u8]) -> Option<PathBuf> {
        let dir = self.dir.as_ref()?;
        make_folder(dir).ok()?;
        let name = new_name(&self.id);
        let temp = dir.join(format!(".{name}.tmp"));
        let path = dir.join(&name);
        let written = write_new(&temp, bytes).and_then(|()| fs::rename(&temp, &path));
        if written.is_err() {
            let _ = fs::remove_file(&temp);
            return None;
        }
        Some(path)
    }

    /// The copies left by earlier runs, oldest first. A file that cannot be
    /// read stays where it is.
    pub(super) fn waiting(&self) -> Vec<SentCopy> {
        let Some(dir) = &self.dir else {
            return Vec::new();
        };
        let Ok(entries) = fs::read_dir(dir) else {
            return Vec::new();
        };
        let mine = format!("{}.", self.id);
        let mut files: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == EXTENSION))
            .filter(|path| path.file_name().is_some_and(|name| name.to_string_lossy().starts_with(&mine)))
            .collect();
        files.sort();
        files
            .into_iter()
            .filter_map(|file| fs::read(&file).ok().map(|bytes| SentCopy { bytes, file: Some(file), uncertain: true }))
            .collect()
    }
}

/// Takes a copy off the disk once the server has it, or has no place for it.
pub(super) fn forget(file: &Path) {
    let _ = fs::remove_file(file);
}

/// A name that starts with the account and then sorts by the time the copy
/// was made, and is unique within this process and, with the process number,
/// across runs.
fn new_name(id: &str) -> String {
    static COUNT: AtomicU32 = AtomicU32::new(0);
    let count = COUNT.fetch_add(1, Ordering::Relaxed);
    format!("{id}.{:012}-{}-{count:04}.{EXTENSION}", now_unix().max(0), std::process::id())
}

#[cfg(test)]
include!("sent_queue.tests.rs");
