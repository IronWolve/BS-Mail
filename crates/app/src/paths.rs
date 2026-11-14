//! Where the program's own files are. Every path is resolved from the
//! executable's own location, never from the working directory, so the folder
//! holding the program can be named or renamed anything.

use std::path::PathBuf;

/// The mock fixture installed beside the program.
pub const FIXTURE_NAME: &str = "mock.toml";

/// The folder the program keeps what it remembers in, beside the program.
pub const DATA_DIR_NAME: &str = "data";

/// The account file installed beside the program. When it is there, the
/// program reads that account's real mail instead of the mock.
pub const ACCOUNT_NAME: &str = "account.toml";

/// The folder the running executable sits in.
pub fn program_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe = exe.canonicalize().unwrap_or(exe);
    exe.parent().map(PathBuf::from)
}

/// Where the program keeps what it remembers: the folder asked for, else a
/// `data` folder beside the program. Nothing is created until something is
/// saved.
pub fn data_dir(asked: Option<&PathBuf>) -> Option<PathBuf> {
    match asked {
        Some(path) => Some(path.clone()),
        None => program_dir().map(|dir| dir.join(DATA_DIR_NAME)),
    }
}

/// The account file beside the program, when one is there.
pub fn installed_account() -> Option<PathBuf> {
    let path = program_dir()?.join(ACCOUNT_NAME);
    path.is_file().then_some(path)
}

/// The mock fixture beside the program, when one is installed there.
pub fn installed_fixture() -> Option<PathBuf> {
    let path = program_dir()?.join(FIXTURE_NAME);
    path.is_file().then_some(path)
}
