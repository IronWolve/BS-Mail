//! Where the program's own files are. Every path is resolved from the
//! executable's own location, never from the working directory, so the folder
//! holding the program can be named or renamed anything.

use std::path::PathBuf;

/// The mock fixture installed beside the program.
pub const FIXTURE_NAME: &str = "mock.toml";

/// The folder the running executable sits in.
pub fn program_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe = exe.canonicalize().unwrap_or(exe);
    exe.parent().map(PathBuf::from)
}

/// The mock fixture beside the program, when one is installed there.
pub fn installed_fixture() -> Option<PathBuf> {
    let path = program_dir()?.join(FIXTURE_NAME);
    path.is_file().then_some(path)
}
