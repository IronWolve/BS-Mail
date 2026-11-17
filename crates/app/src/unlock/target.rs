//! Which vault a run uses and which data folder it holds, worked out from the
//! command line alone, before the accounts are looked at: the gate needs them
//! before the screen is built.

use std::path::PathBuf;

use crate::accounts::Place;
use crate::cli::Options;
use crate::paths::{accounts_dir, data_dir};

/// The folder the vault and its key file are in for this run: `secrets` beside
/// the account file of `--account`, else `secrets` in the data folder. `None`
/// for mock mail, which has no vault.
pub fn vault_folder(options: &Options) -> Option<PathBuf> {
    if let Some(file) = &options.account {
        return Some(Place::File(file.clone()).secrets_folder());
    }
    if options.mock.is_some() && !options.setup {
        return None;
    }
    let data = data_dir(options.data_dir.as_ref())?;
    Some(Place::Root(accounts_dir(&data)).secrets_folder())
}

/// The data folder this run holds the lock on: only a data folder of its own,
/// never the folder of an `--account` file, which other runs may share.
pub fn data_folder(options: &Options) -> Option<PathBuf> {
    if options.account.is_some() || (options.mock.is_some() && !options.setup) {
        return None;
    }
    data_dir(options.data_dir.as_ref())
}

#[cfg(test)]
include!("target.tests.rs");
