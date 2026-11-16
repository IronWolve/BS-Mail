//! The attachments saved from messages: files in `attachments/` in the data
//! folder, in a folder only its owner can open, each readable by its owner
//! only. A file is always a new file: its name is the sender's name made safe
//! (see `attachment_name`), a name already taken gets ` (2)`, ` (3)`... before
//! its extension, and one that is there is never written over or written
//! through (a new file is created exclusively, which also refuses a link left
//! in the way). A save that fails at any point, a full disk included, removes
//! what it had written, so no half file is ever left.
//!
//! "Save as" puts one file where the user typed (see `save_to`), under the
//! same rules: new file, owner only, no partial file.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::attachment_name::{numbered, safe_name};

/// The folder inside the data folder that holds the saved attachments.
pub const ATTACHMENTS_DIR_NAME: &str = "attachments";

/// How many names are tried before the save gives up on finding a free one.
const MAX_TRIES: u32 = 1000;

/// A file that was saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Saved {
    pub path: PathBuf,
    /// The name it was saved under.
    pub name: String,
}

/// The folder of saved attachments of one data folder.
#[derive(Debug, Clone)]
pub struct AttachmentStore {
    dir: PathBuf,
}

impl AttachmentStore {
    /// The store of the data folder `data`. Nothing is made until a file is
    /// saved.
    pub fn open(data: &Path) -> Self {
        Self { dir: data.join(ATTACHMENTS_DIR_NAME) }
    }

    /// The folder the attachments are saved in.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Saves `data` as a new file in the folder, named after `name` (the
    /// sender's) and `content_type`.
    pub fn save(&self, name: &str, content_type: &str, data: &[u8]) -> io::Result<Saved> {
        crate::private_dirs::make(&self.dir)?;
        let kind = fs::symlink_metadata(&self.dir)?.file_type();
        if !kind.is_dir() {
            return Err(io::Error::other("the attachments folder is not a plain folder (it is a link or a file)"));
        }
        save_into(&self.dir, name, content_type, data)
    }
}

/// Saves `data` as a new file in `dir`, which is there already, named after
/// `name` (the sender's, made safe) and `content_type`; a name already taken
/// gets a number before its extension.
pub fn save_into(dir: &Path, name: &str, content_type: &str, data: &[u8]) -> io::Result<Saved> {
    let wanted = safe_name(name, content_type);
    for attempt in 1..=MAX_TRIES {
        let candidate = if attempt == 1 { wanted.clone() } else { numbered(&wanted, attempt) };
        match write_new(&dir.join(&candidate), data) {
            Ok(()) => return Ok(Saved { path: dir.join(&candidate), name: candidate }),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "no free name was found for the file"))
}

/// Saves `data` as a new file at exactly `target`: its folder must be there,
/// and a file that is already there is not touched.
pub fn save_to(target: &Path, data: &[u8]) -> io::Result<()> {
    write_new(target, data)
}

/// Creates `path` as a new file for its owner only, writes `data`, and makes
/// sure it reached the disk. On any failure the file this call created is
/// removed again.
fn write_new(path: &Path, data: &[u8]) -> io::Result<()> {
    let file = create_private(path)?;
    fill_or_remove(file, path, data)
}

/// Writes `data` to `file`, which is the new file at `path`; when that fails
/// the file at `path` is removed.
fn fill_or_remove(mut file: fs::File, path: &Path, data: &[u8]) -> io::Result<()> {
    let written = file.write_all(data).and_then(|()| file.flush()).and_then(|()| file.sync_all());
    drop(file);
    if let Err(error) = written {
        // This is the file just created above, and only it.
        let _ = fs::remove_file(path);
        return Err(error);
    }
    Ok(())
}

#[cfg(unix)]
fn create_private(path: &Path) -> io::Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)
}

#[cfg(not(unix))]
fn create_private(path: &Path) -> io::Result<fs::File> {
    fs::OpenOptions::new().write(true).create_new(true).open(path)
}

/// What went wrong with a file, in words for the footer.
pub fn plain_words(error: &io::Error) -> String {
    match error.kind() {
        io::ErrorKind::NotFound => "the folder is not there".to_owned(),
        io::ErrorKind::PermissionDenied => "permission was denied".to_owned(),
        io::ErrorKind::AlreadyExists => "a file with that name is already there".to_owned(),
        io::ErrorKind::StorageFull => "the disk is full".to_owned(),
        io::ErrorKind::NotADirectory => "part of that path is a file, not a folder".to_owned(),
        _ => error.to_string(),
    }
}

#[cfg(test)]
include!("attachment_store.tests.rs");
