//! The token file: the access token, the refresh token and when the access
//! token ends, as TOML. It is written to a temporary name in the same folder
//! with owner-only permissions and then renamed into place, so a reader never
//! sees half a file; on Unix a file that others can read is refused.

use std::io::Write;
use std::path::{Path, PathBuf};

use model::Secret;
use serde::{Deserialize, Serialize};

use crate::send::random;

/// The tokens of one account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenRecord {
    pub access_token: Secret,
    pub refresh_token: Secret,
    /// Unix seconds at which the access token stops working.
    pub expires_at: i64,
}

/// Why the file could not be read or written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileError {
    /// There is no file: the account has not been signed in.
    Missing,
    /// Another problem, as one sentence.
    Failed(String),
}

#[derive(Serialize, Deserialize)]
struct OnDisk {
    access_token: String,
    refresh_token: String,
    expires_at: i64,
}

/// Reads the token file at `path`.
pub fn read(path: &Path) -> Result<TokenRecord, FileError> {
    let meta = match std::fs::metadata(path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Err(FileError::Missing),
        Err(error) => return Err(FileError::Failed(format!("cannot read the token file {}: {error}", path.display()))),
    };
    check_private(path, &meta)?;
    let text = std::fs::read_to_string(path)
        .map_err(|error| FileError::Failed(format!("cannot read the token file {}: {error}", path.display())))?;
    let parsed: OnDisk = toml::from_str(&text).map_err(|_| {
        FileError::Failed(format!("the token file {} is not valid; run the program with --setup to sign in again", path.display()))
    })?;
    if parsed.access_token.is_empty() || parsed.refresh_token.is_empty() {
        return Err(FileError::Failed(format!(
            "the token file {} has an empty token; run the program with --setup to sign in again",
            path.display()
        )));
    }
    Ok(TokenRecord {
        access_token: Secret::new(parsed.access_token),
        refresh_token: Secret::new(parsed.refresh_token),
        expires_at: parsed.expires_at,
    })
}

#[cfg(unix)]
fn check_private(path: &Path, meta: &std::fs::Metadata) -> Result<(), FileError> {
    use std::os::unix::fs::PermissionsExt;
    let mode = meta.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(FileError::Failed(format!(
            "the token file {p} holds sign-in tokens and is readable by other users (mode {mode:o}); run: chmod 600 {p}",
            p = path.display()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_private(_path: &Path, _meta: &std::fs::Metadata) -> Result<(), FileError> {
    Ok(())
}

/// Writes `record` to `path`: the folder is made when missing, the text goes
/// to a new owner-only file beside it, and that file is renamed over `path`.
/// On any failure the temporary file is removed and `path` is untouched.
pub fn write(path: &Path, record: &TokenRecord) -> Result<(), FileError> {
    let fail = |what: &str, error: &dyn std::fmt::Display| {
        FileError::Failed(format!("cannot save the sign-in tokens to {}: {what}: {error}", path.display()))
    };
    let dir = path.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    make_folder(dir).map_err(|e| fail("the folder", &e))?;
    let text = toml::to_string(&OnDisk {
        access_token: record.access_token.expose().to_owned(),
        refresh_token: record.refresh_token.expose().to_owned(),
        expires_at: record.expires_at,
    })
    .map_err(|e| fail("the text", &e))?;
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let suffix = random::random_hex::<6>().map_err(|e| fail("a temporary name", &e))?;
    let temp: PathBuf = dir.join(format!(".{name}.{suffix}.tmp"));
    let result = write_new(&temp, text.as_bytes()).and_then(|()| std::fs::rename(&temp, path));
    if let Err(error) = result {
        let _ = std::fs::remove_file(&temp);
        return Err(fail("writing the file", &error));
    }
    Ok(())
}

#[cfg(unix)]
fn make_folder(dir: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(dir)
}

#[cfg(not(unix))]
fn make_folder(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)
}

#[cfg(unix)]
fn open_new(path: &Path) -> std::io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)
}

#[cfg(not(unix))]
fn open_new(path: &Path) -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new().write(true).create_new(true).open(path)
}

fn write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = open_new(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(test)]
include!("token_file.tests.rs");
