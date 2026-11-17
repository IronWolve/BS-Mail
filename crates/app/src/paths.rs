//! Where the program's own files are. Every path is resolved from the
//! executable's own location, never from the working directory, so the folder
//! holding the program can be named or renamed anything.

use std::path::{Path, PathBuf};

/// The folder the program keeps what it remembers in, beside the program.
pub const DATA_DIR_NAME: &str = "data";

/// The folder inside the data folder that holds one folder per account.
pub const ACCOUNTS_DIR_NAME: &str = "accounts";

/// The folder inside the data folder that holds the encrypted vault and its
/// key file.
pub const SECRETS_DIR_NAME: &str = "secrets";

/// The account file inside an account's own folder.
pub const ACCOUNT_FILE: &str = "account.toml";

/// The file inside the data folder that one running copy holds a lock on.
pub const LOCK_FILE: &str = "lock";

/// The folder the running executable sits in.
pub fn program_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe = exe.canonicalize().unwrap_or(exe);
    exe.parent().map(PathBuf::from)
}

/// Where the program keeps what it remembers: the folder asked for, else a
/// `data` folder beside the program. A run that uses it makes it and its lock
/// file; nothing else is created until something is saved.
pub fn data_dir(asked: Option<&PathBuf>) -> Option<PathBuf> {
    match asked {
        Some(path) => Some(path.clone()),
        None => program_dir().map(|dir| dir.join(DATA_DIR_NAME)),
    }
}

/// The folder that holds the accounts, inside the data folder.
pub fn accounts_dir(data: &Path) -> PathBuf {
    data.join(ACCOUNTS_DIR_NAME)
}

#[cfg(test)]
include!("paths.tests.rs");
