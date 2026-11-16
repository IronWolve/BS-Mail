//! The address book on disk: `addresses.txt` in the data folder, one
//! `Name <address>` per line, most recent first, readable by its owner only.
//!
//! It is only what the program has seen, kept to complete recipients as they
//! are typed (the book and its rules are `ui::AddressBook`). A file that is
//! missing, unreadable or not understood gives an empty book and is never an
//! error: the worst of a damaged file is that nothing completes until mail has
//! been loaded again. Nothing in it is a path.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use ui::AddressBook;

pub const ADDRESS_FILE_NAME: &str = "addresses.txt";

/// The most of the file that is read. A book of the most addresses it keeps
/// is a small fraction of this, so a bigger file is not one of ours.
const MAX_READ_BYTES: u64 = 1 << 20;

pub fn path_in(dir: &Path) -> PathBuf {
    dir.join(ADDRESS_FILE_NAME)
}

/// The book the data folder holds; an empty one when there is none.
pub fn load(dir: &Path) -> AddressBook {
    let Ok(file) = fs::File::open(path_in(dir)) else {
        return AddressBook::default();
    };
    let mut bytes = Vec::new();
    if file.take(MAX_READ_BYTES).read_to_end(&mut bytes).is_err() {
        return AddressBook::default();
    }
    AddressBook::from_text(&String::from_utf8_lossy(&bytes))
}

/// Writes the book's text through a temporary file and a rename, so a crash
/// never leaves half a file, and the file is private from the moment it
/// exists: it holds the names and addresses of the people the user writes to.
/// Makes the data folder, privately, when it is not there yet.
pub fn save(dir: &Path, text: &str) -> io::Result<()> {
    crate::private_dirs::make(dir)?;
    let target = path_in(dir);
    let temporary = dir.join(format!("{ADDRESS_FILE_NAME}.part"));
    // A leftover of an earlier try is removed (a link is removed itself, never
    // followed), so nothing a link points at is ever written.
    if fs::symlink_metadata(&temporary).is_ok() {
        fs::remove_file(&temporary)?;
    }
    let mut file = open_private(&temporary)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temporary, &target)
}

#[cfg(unix)]
fn open_private(path: &Path) -> io::Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)
}

#[cfg(not(unix))]
fn open_private(path: &Path) -> io::Result<fs::File> {
    fs::OpenOptions::new().write(true).create_new(true).open(path)
}

#[cfg(test)]
include!("address_file.tests.rs");
