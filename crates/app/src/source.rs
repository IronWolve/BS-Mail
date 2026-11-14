//! Where the mail comes from: a real account, a mock file asked for by name,
//! or nothing yet, in which case the account form is shown. Mock mail is never
//! picked up by itself: a program with no account starts empty.

use std::path::PathBuf;

use crate::cli::Options;
use crate::errors::AppError;
use crate::paths::{account_target, installed_account};

/// The mail source for this run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Mock(PathBuf),
    Account(PathBuf),
    /// No account yet: the form asks for one and saves it at this path.
    Setup(PathBuf),
}

impl Source {
    pub fn path(&self) -> &PathBuf {
        match self {
            Self::Mock(path) | Self::Account(path) | Self::Setup(path) => path,
        }
    }
}

/// What was asked for outright wins; otherwise an account file beside the
/// program, and without one the account form, which saves to `setup_target`.
/// Both are looked up by the caller so the rule can be checked without
/// touching the file system.
pub fn choose(
    options: &Options,
    account_beside: Option<PathBuf>,
    setup_target: Option<PathBuf>,
) -> Result<Source, AppError> {
    if options.setup {
        return options
            .account
            .clone()
            .or(setup_target)
            .map(Source::Setup)
            .ok_or(AppError::NoSource);
    }
    if let Some(path) = &options.mock {
        return Ok(Source::Mock(path.clone()));
    }
    if let Some(path) = &options.account {
        return Ok(Source::Account(path.clone()));
    }
    account_beside
        .map(Source::Account)
        .or_else(|| setup_target.map(Source::Setup))
        .ok_or(AppError::NoSource)
}

/// `choose` with the program's own folder looked at.
pub fn choose_for_run(options: &Options) -> Result<Source, AppError> {
    choose(options, installed_account(), account_target())
}

#[cfg(test)]
include!("source.tests.rs");
