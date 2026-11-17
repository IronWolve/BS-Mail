//! New tokens that could not be saved to the vault yet.
//!
//! A refresh can bring a new refresh token, and the provider may end the old
//! one at that moment. So when the save fails (disk full, a folder that went
//! read-only, a vault that cannot be opened) the new tokens are kept here, in
//! memory, until a save works. While they are here the token source uses them,
//! saves them first, and never refreshes again with the old refresh token in
//! the vault.
//!
//! A crash in this window still loses them; that cannot be helped without
//! writing them somewhere, which is the very thing that failed.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use super::token_place::TokenPlace;
use super::token_store::TokenRecord;

/// The unsaved tokens, one entry per account (its vault folder and id).
static UNSAVED: Mutex<Vec<((PathBuf, String), TokenRecord)>> = Mutex::new(Vec::new());

fn all() -> MutexGuard<'static, Vec<((PathBuf, String), TokenRecord)>> {
    UNSAVED.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Keeps `record` as the newest tokens of the account at `place`, replacing an
/// earlier one.
pub(super) fn keep(place: &TokenPlace, record: TokenRecord) {
    let key = place.key();
    let mut held = all();
    held.retain(|(kept, _)| *kept != key);
    held.push((key, record));
}

/// The tokens kept for `place`, if a save has not yet worked.
pub(super) fn waiting(place: &TokenPlace) -> Option<TokenRecord> {
    let key = place.key();
    all().iter().find(|(kept, _)| *kept == key).map(|(_, record)| record.clone())
}

/// Lets go of what is kept for `place`: it was saved, or a newer sign-in
/// replaced it.
pub(super) fn forget(place: &TokenPlace) {
    let key = place.key();
    all().retain(|(kept, _)| *kept != key);
}
