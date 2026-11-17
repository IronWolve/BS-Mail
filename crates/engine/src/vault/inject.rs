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
}

#[cfg(test)]
thread_local! {
    static FAIL_AT: std::cell::Cell<Option<Step>> = const { std::cell::Cell::new(None) };
}

/// Makes the next arrival at `step` on this thread fail, until called again
/// with `None`.
#[cfg(test)]
pub(super) fn fail_at(step: Option<Step>) {
    FAIL_AT.with(|cell| cell.set(step));
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
