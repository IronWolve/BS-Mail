//! Named points in a save and in a switch of protection where a test can make
//! the step fail, to prove the old vault and key survive a failure at each.
//! Outside tests `reach` does nothing at all.

use super::error::VaultError;

/// The steps of writing the vault and changing how it is protected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Step {
    /// Making the key from a new passphrase.
    Derive,
    /// Writing a new key file beside the old one.
    WriteKey,
    /// Putting that key file in its place.
    RenameKey,
    /// Writing the new vault under its temporary name.
    WriteTemp,
    /// Renaming the new vault over the old one.
    Rename,
    /// Removing the key file the old vault used.
    RemoveKey,
    /// Flushing the folder after the new vault was renamed into it.
    SyncVault,
    /// Flushing the folder after a new key file was renamed into it.
    SyncKey,
    /// Writing the word that records how the vault is protected.
    WriteMarker,
    /// Tightening the permissions of the folder or a file in it (Unix only).
    #[cfg_attr(not(unix), allow(dead_code))]
    Tighten,
}

#[cfg(test)]
thread_local! {
    static FAIL_AT: std::cell::Cell<Option<Step>> = const { std::cell::Cell::new(None) };
    static NO_LOCKS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Makes the next arrival at `step` on this thread fail, until called again
/// with `None`.
#[cfg(test)]
pub(super) fn fail_at(step: Option<Step>) {
    FAIL_AT.with(|cell| cell.set(step));
}

/// Makes every attempt to lock the secrets folder on this thread answer as a
/// file system with no locks would, until called again with `false`.
#[cfg(test)]
pub(super) fn no_locks(on: bool) {
    NO_LOCKS.with(|cell| cell.set(on));
}

/// A point a step passes through; in a test it can be made to fail.
pub(super) fn reach(step: Step) -> Result<(), VaultError> {
    #[cfg(test)]
    if FAIL_AT.with(std::cell::Cell::get) == Some(step) {
        return Err(VaultError::Write("a failure made by a test".to_string()));
    }
    let _ = step;
    Ok(())
}

/// What a file system with no locks would answer, when a test asks for it.
pub(super) fn lock_fault() -> Option<std::io::Error> {
    #[cfg(test)]
    if NO_LOCKS.with(std::cell::Cell::get) {
        return Some(std::io::Error::from(std::io::ErrorKind::Unsupported));
    }
    None
}
