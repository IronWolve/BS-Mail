//! The token file an older version wrote, `oauth/<id>.token` beside the
//! account file: the access token, the refresh token and when the access token
//! ends, as TOML. Nothing writes one any more: it is read when the vault has no
//! sign-in for the account, and removed once the tokens are in the vault. On
//! Unix a file that others can read is refused, as it always was.

use std::path::Path;

use model::Secret;
use serde::Deserialize;

use super::token_store::{StoreError, TokenRecord};
use crate::sign_in_advice::sign_in_again;

#[derive(Deserialize)]
struct OnDisk {
    access_token: String,
    refresh_token: String,
    expires_at: i64,
}

/// Reads the token file at `path`.
pub fn read(path: &Path) -> Result<TokenRecord, StoreError> {
    let meta = match std::fs::metadata(path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Err(StoreError::Missing),
        Err(error) => return Err(StoreError::Failed(format!("cannot read the token file {}: {error}", path.display()))),
    };
    check_private(path, &meta)?;
    let text = std::fs::read_to_string(path)
        .map_err(|error| StoreError::Failed(format!("cannot read the token file {}: {error}", path.display())))?;
    let parsed: OnDisk = toml::from_str(&text).map_err(|_| {
        StoreError::Failed(format!("the token file {} is not valid; {}", path.display(), sign_in_again()))
    })?;
    if parsed.access_token.is_empty() || parsed.refresh_token.is_empty() {
        return Err(StoreError::Failed(format!(
            "the token file {} has an empty token; {}",
            path.display(),
            sign_in_again()
        )));
    }
    Ok(TokenRecord {
        access_token: Secret::new(parsed.access_token),
        refresh_token: Secret::new(parsed.refresh_token),
        expires_at: parsed.expires_at,
    })
}

#[cfg(unix)]
fn check_private(path: &Path, meta: &std::fs::Metadata) -> Result<(), StoreError> {
    use std::os::unix::fs::PermissionsExt;
    let mode = meta.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(StoreError::Failed(format!(
            "the token file {p} holds sign-in tokens and is readable by other users (mode {mode:o}); run: chmod 600 {p}",
            p = path.display()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_private(_path: &Path, _meta: &std::fs::Metadata) -> Result<(), StoreError> {
    Ok(())
}

/// Removes the token file at `path` and the `oauth` folder it was in when that
/// is empty now. A file that is not there is nothing to do; a failure is not
/// reported here (the start-up move says what is left, with its name).
pub fn remove(path: &Path) {
    if path.as_os_str().is_empty() {
        return;
    }
    if std::fs::remove_file(path).is_ok()
        && let Some(folder) = path.parent()
    {
        let _ = std::fs::remove_dir(folder);
    }
}

#[cfg(test)]
include!("token_file.tests.rs");
