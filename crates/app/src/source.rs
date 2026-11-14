//! Where the mail comes from: mock mail asked for by name, one account file
//! asked for by name, the accounts kept in the data folder, or nothing yet, in
//! which case the account form is shown. Mock mail is never picked up by
//! itself: a program with no account starts empty.

use std::path::PathBuf;

use crate::accounts::Place;
use crate::cli::Options;
use crate::errors::AppError;

/// The mail source for this run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// `--mock <file>`: mail from a fixture, nothing real.
    Mock(PathBuf),
    /// `--account <file>`: exactly that one account.
    Account(PathBuf),
    /// The accounts folder holds at least one account.
    Accounts(PathBuf),
    /// The accounts folder holds none: the program starts empty with the form
    /// open, and a new account is saved there.
    Empty(PathBuf),
    /// `--setup --account <file>`: the form saves exactly that file, and no
    /// account runs until it has.
    Setup(PathBuf),
}

impl Source {
    /// Whether this source has nothing to show until the form has made an
    /// account.
    pub fn starts_empty(&self) -> bool {
        matches!(self, Self::Empty(_) | Self::Setup(_))
    }

    /// The data folder this run keeps its accounts in, which one copy at a
    /// time may use. `None` for mock mail, which keeps nothing, and for
    /// `--account`, the scripted way of running: it names one file, which may
    /// sit in a folder shared with other files and other runs, so it is not a
    /// data folder of its own.
    pub fn data_folder(&self) -> Option<PathBuf> {
        match self {
            Self::Mock(_) | Self::Account(_) | Self::Setup(_) => None,
            Self::Accounts(root) | Self::Empty(root) => Some(Place::Root(root.clone()).data_folder()),
        }
    }
}

/// Whether the accounts folder has to be looked at to choose: not when the
/// mail is named outright, unless the form was asked for too.
pub fn needs_scan(options: &Options) -> bool {
    options.account.is_none() && (options.mock.is_none() || options.setup)
}

/// What was asked for outright wins; otherwise the accounts in `root`, the
/// folder under the data folder, of which `found` were there. Without any the
/// source is empty and the form is shown. `--setup` over accounts keeps them
/// running, and `--setup` with an account file ignores the accounts folder.
pub fn choose(options: &Options, root: Option<PathBuf>, found: usize) -> Result<Source, AppError> {
    if options.setup
        && let Some(file) = &options.account
    {
        return Ok(Source::Setup(file.clone()));
    }
    if !options.setup {
        if let Some(path) = &options.mock {
            return Ok(Source::Mock(path.clone()));
        }
        if let Some(path) = &options.account {
            return Ok(Source::Account(path.clone()));
        }
    }
    let root = root.ok_or(AppError::NoSource)?;
    Ok(if found > 0 { Source::Accounts(root) } else { Source::Empty(root) })
}

#[cfg(test)]
include!("source.tests.rs");
