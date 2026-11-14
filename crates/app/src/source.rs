//! Where the mail comes from: the mock, a real account, or nothing yet, in
//! which case the account form is shown.

use std::path::PathBuf;

use crate::cli::Options;
use crate::errors::AppError;
use crate::paths::{account_target, installed_account, installed_fixture};

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
/// program, then a mock file beside the program, and with neither the
/// account form, which saves to `setup_target`. All are looked up by the
/// caller so the rule can be checked without touching the file system.
pub fn choose(
    options: &Options,
    account_beside: Option<PathBuf>,
    mock_beside: Option<PathBuf>,
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
        .or_else(|| mock_beside.map(Source::Mock))
        .or_else(|| setup_target.map(Source::Setup))
        .ok_or(AppError::NoSource)
}

/// `choose` with the program's own folder looked at.
pub fn choose_for_run(options: &Options) -> Result<Source, AppError> {
    choose(options, installed_account(), installed_fixture(), account_target())
}

#[cfg(test)]
include!("source.tests.rs");
