//! Why a setting could not be saved, in one line for the Options screen. The
//! lines are short on purpose: the screen cuts a line at its width, and what
//! the user most needs is at the front.

use std::fmt;
use std::io;

use crate::config_io::CONFIG_NAME;

/// A setting that could not be written to `config.toml`. The file's own
/// text is never part of the words.
#[derive(Debug)]
pub enum SaveProblem {
    /// There is no data folder to keep the file in.
    NoFolder,
    /// The folder or the file could not be read, or the new file written.
    Io(io::Error),
    /// The file is not valid TOML; the first line that is wrong. The file was
    /// not touched.
    NotToml { line: usize },
    /// A table the setting lives in is something else in the file (a number,
    /// a string). The file was not touched.
    NotATable(&'static str),
}

impl fmt::Display for SaveProblem {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoFolder => write!(out, "could not save {CONFIG_NAME}: there is no data folder"),
            Self::Io(error) => write!(out, "could not save {CONFIG_NAME}: {error}"),
            Self::NotToml { line } => write!(out, "{CONFIG_NAME} is not valid TOML (line {line}) and was left as it is"),
            Self::NotATable(name) => write!(out, "[{name}] in {CONFIG_NAME} is not a table and the file was left as it is"),
        }
    }
}

impl std::error::Error for SaveProblem {}

impl From<io::Error> for SaveProblem {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
