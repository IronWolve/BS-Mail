//! Everything that can stop a run, in the words the user sees.

use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::paths::{ACCOUNT_NAME, FIXTURE_NAME};

#[derive(Debug)]
pub enum AppError {
    NoSource,
    Fixture(PathBuf, String),
    Account(PathBuf, String),
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
                "no mail to show: put {ACCOUNT_NAME} or {FIXTURE_NAME} beside the program, \
                 or use --account <file> or --mock <file>"
            ),
            Self::Account(path, reason) => {
                write!(f, "the account file {} cannot be used: {reason}", path.display())
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
