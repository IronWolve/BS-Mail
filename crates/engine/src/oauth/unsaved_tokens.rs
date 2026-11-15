//! New tokens that could not be written to the token file yet.
//!
//! A refresh can bring a new refresh token, and the provider may end the old
//! one at that moment. So when the write fails (disk full, a folder that went
//! read-only) the new tokens are kept here, in memory, until a write works.
//! While they are here the token source uses them, saves them first, and
//! never refreshes again with the old refresh token on the file.
//!
//! A crash in this window still loses them; that cannot be helped without
//! writing them somewhere, which is the very thing that failed.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use super::token_file::TokenRecord;

/// The unsaved tokens, one entry per token file.
static UNSAVED: Mutex<Vec<(PathBuf, TokenRecord)>> = Mutex::new(Vec::new());

fn all() -> MutexGuard<'static, Vec<(PathBuf, TokenRecord)>> {
    UNSAVED.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Keeps `record` as the newest tokens of `path`, replacing an earlier one.
pub(super) fn keep(path: &Path, record: TokenRecord) {
    let mut held = all();
    held.retain(|(kept, _)| kept != path);
    held.push((path.to_path_buf(), record));
}

/// The tokens kept for `path`, if a write has not yet worked.
pub(super) fn waiting(path: &Path) -> Option<TokenRecord> {
    all().iter().find(|(kept, _)| kept == path).map(|(_, record)| record.clone())
}

/// Lets go of what is kept for `path`: it was saved, or a newer sign-in
/// replaced it.
pub(super) fn forget(path: &Path) {
    all().retain(|(kept, _)| kept != path);
}
