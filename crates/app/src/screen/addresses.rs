//! Keeping the address book: the addresses the screen has seen in lists and
//! written to are saved in the data folder when they change.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::Screen;
use crate::address_file;

/// How long after a save that failed the next one is tried, so a folder that
/// cannot be written is not tried on every tick.
const RETRY_AFTER: Duration = Duration::from_secs(30);

/// Where the book is kept, and when a save that failed may be tried again.
pub(super) struct AddressSave {
    dir: PathBuf,
    retry_at: Option<Instant>,
}

impl Screen {
    /// Keeps the address book in `dir`: what is saved there is already in the
    /// view's book (the scene read it), and from here on a change is written.
    pub fn keep_addresses_in(mut self, dir: PathBuf) -> Self {
        self.addresses = Some(AddressSave { dir, retry_at: None });
        self
    }

    /// Writes the book when it changed since the last write. A write that
    /// fails is tried again after a while and at the end of the run, and
    /// costs nothing but the completions of the next run.
    pub(super) fn keep_addresses(&mut self) {
        let Some(keep) = self.addresses.as_mut() else {
            return;
        };
        if keep.retry_at.is_some_and(|at| Instant::now() < at) {
            return;
        }
        let Some(text) = self.view.take_address_book_text() else {
            return;
        };
        match address_file::save(&keep.dir, &text) {
            Ok(()) => keep.retry_at = None,
            Err(_) => {
                self.view.address_book_unsaved();
                keep.retry_at = Some(Instant::now() + RETRY_AFTER);
            }
        }
    }

    /// The last write of the run: whatever changed and was not written yet.
    pub fn save_addresses(&mut self) {
        if let Some(keep) = self.addresses.as_mut() {
            keep.retry_at = None;
        }
        self.keep_addresses();
    }
}

#[cfg(test)]
include!("addresses.tests.rs");
