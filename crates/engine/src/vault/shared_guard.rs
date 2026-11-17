//! What the shared handle does before it touches the files of a change: looks
//! at the secrets folder (a real folder, owner-only), takes the folder's lock
//! shared with the other copies of the program, and keeps what it has to say
//! (permissions were tightened) until the program takes it.

use super::error::VaultError;
use super::file_lock::{self, FileLock};
use super::format::Mode;
use super::key_source::KeySource;
use super::marker::{self, Marker};
use super::permissions;
use super::shared::{Held, SharedVault};

impl SharedVault {
    /// Checks the secrets folder and what is in it, tightening permissions
    /// that let others in and noting that it did (said once: after it, nothing
    /// is loose). A link in place of the folder, or permissions that cannot be
    /// tightened, are errors.
    pub(super) fn open_folder(&self) -> Result<(), VaultError> {
        let said = permissions::check(&self.dir)?;
        let mut notes = self.notes.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        for note in said {
            if !notes.iter().any(|known| known == note) {
                notes.push(note.to_owned());
            }
        }
        Ok(())
    }

    /// What the folder's checks had to say since the last call, for the
    /// footer: the program takes it once, after it has opened the vault.
    pub fn take_notes(&self) -> Vec<String> {
        std::mem::take(&mut *self.notes.lock().unwrap_or_else(std::sync::PoisonError::into_inner))
    }

    /// The start of a change that reads and writes files: the folder is
    /// checked, a keychain vault's key is read now if nothing read it yet (so
    /// a keychain that waits is never waited for with the folder's lock held),
    /// the folder's lock is taken (`create`: made first, for a change that
    /// will write; `None` when the folder is not there yet or cannot be
    /// written to), and the key source is worked out under it, as the file
    /// says it is now.
    pub(super) fn begin(
        &self,
        held: &mut Held,
        create: bool,
    ) -> Result<(Option<FileLock>, Result<KeySource, VaultError>), VaultError> {
        self.open_folder()?;
        let _ = self.source(held);
        let lock = file_lock::acquire(&self.dir, create)?;
        Ok((lock, self.source(held)))
    }

    /// Makes the folder's word say `mode` when it does not (the header is the
    /// truth while it can be read; this repairs a word that is missing or
    /// stale). Best effort: a failure changes nothing about the vault.
    pub(super) fn repair_marker(&self, mode: Mode) {
        if marker::read(&self.dir) == Marker::Valid(mode) {
            return;
        }
        if let Ok(_lock) = file_lock::acquire(&self.dir, false) {
            let _ = marker::write(&self.dir, mode);
        }
    }
}

#[cfg(test)]
include!("shared_guard.tests.rs");
