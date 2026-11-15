//! Everything that can stop a run, in the words the user sees.

use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum AppError {
    NoSource,
    Fixture(PathBuf, String),
    Account(PathBuf, String),
    /// Another copy of the program is using this data folder; the text says
    /// who, when it could be told.
    Locked(PathBuf, String),
    /// The data folder can be written but cannot be locked, so a second copy
    /// could not be kept out.
    Unlockable(PathBuf, String),
    Theme(String),
    Fonts(String),
    Output(io::Error),
    Picture(String),
    Window(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSource => write!(
                f,
                "no folder to keep accounts in: use --data-dir <folder>, \
                 or --account <file> or --mock <file>"
            ),
            Self::Account(path, reason) => {
                write!(f, "the account file {} cannot be used: {reason}", path.display())
            }
            Self::Unlockable(folder, reason) => write!(
                f,
                "the data folder cannot be locked ({reason}), so two copies could damage each other's \
                 files; fix the folder, or start this one with --data-dir <another folder> (this folder: {})",
                folder.display()
            ),
            Self::Locked(folder, holder) => {
                let who = if holder.is_empty() { String::new() } else { format!(" ({holder})") };
                write!(
                    f,
                    "another copy is already using the data folder{who}; close it first, \
                     or start this one with --data-dir <another folder> (this folder: {})",
                    folder.display()
                )
            }
            Self::Fixture(path, reason) => {
                write!(f, "the mock file {} did not load: {reason}", path.display())
            }
            Self::Theme(reason) => write!(f, "the built-in theme did not load: {reason}"),
            Self::Fonts(reason) => write!(f, "the built-in fonts did not load: {reason}"),
            Self::Output(error) => write!(f, "writing the screen failed: {error}"),
            Self::Picture(reason) => write!(f, "{reason}"),
            Self::Window(reason) => write!(f, "the window could not run: {reason}"),
        }
    }
}

impl std::error::Error for AppError {}
