//! Reading a file the user named for a message they are writing. The path is
//! what the user typed and nothing else; the file is read once, with the
//! checks made on the open file and not on its name:
//!
//! - only a regular file is read (a folder, a pipe, a device such as
//!   `/dev/zero` and a socket are refused before any read);
//! - its size is checked against what the message has room for before it is
//!   read, and the read is bounded, so a file that grows or is endless costs
//!   no more than the room that is left;
//! - a file inside this program's own data folder is refused, except the
//!   saved attachments: the account files and sign-in tokens there hold
//!   passwords, and the settings and address book are private;
//! - the name sent is the last part of the typed path made safe, and the
//!   content type comes from the extension only, never from what is inside.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use model::attachments::megabytes;
use model::media_types::type_for_name;
use model::{FileBytes, MAX_SEND_BYTES, OutgoingAttachment, TEXT_RESERVE, max_raw_len};

use crate::attachment_name::safe_name;
use crate::attachment_store::ATTACHMENTS_DIR_NAME;

/// What the reading needs to know about this computer and this run.
#[derive(Debug, Clone, Default)]
pub struct Places {
    /// The user's home folder: `~` stands for it and a path that is not
    /// complete starts from it.
    pub home: Option<PathBuf>,
    /// The program's data folder, which holds secrets.
    pub data: Option<PathBuf>,
    /// Other files that hold secrets (the account file named on the command
    /// line, which may live anywhere).
    pub secrets: Vec<PathBuf>,
}

/// The words for a file in the data folder.
const SECRET_WORDS: &str = "that file is in the program's own data folder, which holds passwords, so it is never attached";

/// The file at `typed`, ready to send, or why not. `already` is how much of
/// the message the files attached so far take once encoded.
pub fn read_for_attach(typed: &str, already: u64, places: &Places) -> Result<OutgoingAttachment, String> {
    let path = resolve(typed, places.home.as_deref())?;
    // A look at the name, which never opens it: a pipe would block the open.
    let seen = fs::metadata(&path).map_err(|error| not_found_words(&error))?;
    refuse_unless_regular(&seen)?;
    let real = fs::canonicalize(&path).map_err(|error| not_found_words(&error))?;
    refuse_secrets(&real, places)?;
    let room = MAX_SEND_BYTES.saturating_sub(TEXT_RESERVE).saturating_sub(already);
    let allowed = max_raw_len(room);
    if allowed == 0 {
        return Err(too_full());
    }
    if seen.len() > allowed {
        return Err(too_big(seen.len(), allowed));
    }
    let file = fs::File::open(&real).map_err(|error| not_found_words(&error))?;
    let held = file.metadata().map_err(|error| not_found_words(&error))?;
    refuse_unless_regular(&held)?;
    if !same_file(&seen, &held) {
        return Err("the file changed while it was being opened, so it was not read; try again".to_owned());
    }
    if held.len() > allowed {
        return Err(too_big(held.len(), allowed));
    }
    let mut data = Vec::with_capacity(usize::try_from(held.len()).unwrap_or(0));
    (&file).take(allowed + 1).read_to_end(&mut data).map_err(|error| format!("the file could not be read: {}", crate::attachment_store::plain_words(&error)))?;
    if data.len() as u64 > allowed {
        return Err(too_big(data.len() as u64, allowed));
    }
    let typed_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
    let name = safe_name(typed_name, "");
    let content_type = type_for_name(&name).unwrap_or("application/octet-stream").to_owned();
    Ok(OutgoingAttachment { name, content_type, data: FileBytes::new(data) })
}

/// The path the user typed: blanks and one pair of quotes around it taken off,
/// `~` turned into the home folder, and a path that is not complete started
/// from the home folder.
pub fn resolve(typed: &str, home: Option<&Path>) -> Result<PathBuf, String> {
    let mut text = typed.trim();
    if text.len() >= 2 && ((text.starts_with('"') && text.ends_with('"')) || (text.starts_with('\'') && text.ends_with('\''))) {
        text = text[1..text.len() - 1].trim();
    }
    if text.is_empty() {
        return Err("type the path of a file".to_owned());
    }
    if text.chars().any(char::is_control) {
        return Err("that path has a control character in it, so it was not used".to_owned());
    }
    let no_home = || "this program cannot tell where your home folder is, so type the whole path".to_owned();
    if let Some(rest) = text.strip_prefix('~').filter(|rest| rest.is_empty() || rest.starts_with(['/', '\\'])) {
        let home = home.ok_or_else(no_home)?;
        return Ok(home.join(rest.trim_start_matches(['/', '\\'])));
    }
    let path = Path::new(text);
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(home.ok_or_else(no_home)?.join(path))
}

fn refuse_unless_regular(meta: &fs::Metadata) -> Result<(), String> {
    if meta.is_file() {
        Ok(())
    } else if meta.is_dir() {
        Err("that is a folder, not a file".to_owned())
    } else {
        Err("that is not a regular file (it is a device, a pipe or a socket), so it was not read".to_owned())
    }
}

fn refuse_secrets(real: &Path, places: &Places) -> Result<(), String> {
    let canonical = |path: &Path| fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if let Some(data) = &places.data {
        let data = canonical(data);
        if real.starts_with(&data) && !real.starts_with(data.join(ATTACHMENTS_DIR_NAME)) {
            return Err(SECRET_WORDS.to_owned());
        }
    }
    if places.secrets.iter().any(|secret| canonical(secret) == real) {
        return Err("that file holds an account's password, so it is never attached".to_owned());
    }
    Ok(())
}

fn not_found_words(error: &std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::NotFound => "there is no file at that path".to_owned(),
        std::io::ErrorKind::PermissionDenied => "permission to read that file was denied".to_owned(),
        _ => format!("that file could not be opened: {}", crate::attachment_store::plain_words(error)),
    }
}

fn too_full() -> String {
    format!("the message is already full ({} MB once its files are encoded), so no more files fit", megabytes(MAX_SEND_BYTES))
}

fn too_big(size: u64, allowed: u64) -> String {
    format!(
        "that file is {} MB; the message has room for {} MB of files ({} MB in all once encoded)",
        megabytes(size),
        megabytes(allowed),
        megabytes(MAX_SEND_BYTES),
    )
}

/// Whether two looks are at the same file (where the system says so).
#[cfg(unix)]
fn same_file(first: &fs::Metadata, second: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    first.dev() == second.dev() && first.ino() == second.ino()
}

#[cfg(not(unix))]
fn same_file(_first: &fs::Metadata, _second: &fs::Metadata) -> bool {
    true
}

#[cfg(test)]
include!("attach_read.tests.rs");
