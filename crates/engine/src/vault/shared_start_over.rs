//! Starting the vault over, which only a person's explicit, confirmed choice
//! does: the old vault is set aside (never deleted, never over an earlier set
//! aside one), the folder goes back to "no vault yet, key file", and the next
//! secret given makes a new empty key-file vault. A save never does this for
//! a passphrase or keychain vault (`update_or_start_over`).

use super::aside::{KEY_ASIDE, VAULT_ASIDE, set_aside};
use super::error::VaultError;
use super::format::Mode;
use super::key_source::KEY_FILE_NAME;
use super::marker;
use super::shared::{Held, KeychainHold, SharedVault};
use super::store::{VAULT_FILE_NAME, Vault};

/// What a start-over set aside.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StartedOver {
    /// The name the old vault was kept under in the secrets folder
    /// (`vault.unreadable`, `vault.unreadable.1`...); `None` when there was no
    /// file to keep.
    pub vault_kept_as: Option<String>,
    /// The same for a key file that could not be used.
    pub key_kept_as: Option<String>,
}

impl SharedVault {
    /// Why the vault cannot be opened, when the explicit "Start the vault
    /// over" is the way out (see `VaultError::offers_start_over`): its key or
    /// file is missing, it is damaged or of a kind this program does not know,
    /// or the keychain has no key for it. `None` when it opens, is only locked,
    /// or the keychain is only out of reach. Reads the vault.
    pub fn unopenable(&self) -> Option<VaultError> {
        self.load().err().filter(VaultError::offers_start_over)
    }

    /// Whether a save of a secret given now sets a lost vault aside by itself:
    /// only a key-file vault does (`update_or_start_over`).
    pub fn starts_over_on_save(&self) -> bool {
        self.effective_mode() == Mode::KeyFile
    }

    /// Why a passphrase vault cannot even be asked for its passphrase: the
    /// folder says it is protected by one and the file is missing or its
    /// header cannot be read. `None` for any other vault, and for a passphrase
    /// vault whose header reads (it only needs its passphrase).
    pub fn passphrase_vault_lost(&self) -> Option<VaultError> {
        if self.effective_mode() != Mode::Passphrase {
            return None;
        }
        match self.header() {
            Ok(Some(_)) => None,
            Ok(None) => Some(VaultError::Missing),
            Err(error) => Some(error),
        }
    }

    /// The explicit, confirmed start-over of a vault that cannot be opened
    /// (`unopenable`): the vault file is set aside (so is a key file that
    /// cannot be used), the keys held are dropped and the folder is a key-file
    /// folder again. A vault that opens, is only locked, or whose keychain is
    /// only out of reach is refused with its own sentence and left as it is.
    /// The key of a keychain vault stays in the keychain.
    pub fn start_vault_over(&self) -> Result<StartedOver, VaultError> {
        let mut held = self.hold();
        let (_lock, source) = self.begin(&mut held, false)?;
        let problem = match source.and_then(|source| Vault::open_or_create(&self.dir, source)) {
            Ok(_) => return Err(VaultError::OpensFine),
            Err(error) => error,
        };
        if !problem.offers_start_over() {
            return Err(problem);
        }
        self.set_aside_and_reset(&mut held, matches!(problem, VaultError::BadKeyFile(_)))
    }

    /// For a passphrase that is forgotten, or a passphrase vault that is
    /// missing or damaged: sets the vault file aside as `vault.unreadable`
    /// (nothing is deleted; never over an earlier one) so the folder has no
    /// vault, and drops any key held. The next save makes a new, empty
    /// key-file vault. Only a passphrase vault is set aside.
    pub fn forget_passphrase(&self) -> Result<StartedOver, VaultError> {
        let mut held = self.hold();
        let (_lock, _source) = self.begin(&mut held, false)?;
        if self.effective_mode() != Mode::Passphrase {
            return Err(VaultError::WrongMode);
        }
        self.set_aside_and_reset(&mut held, false)
    }

    /// Moves the vault (and, when it is the bad part, the key file) aside,
    /// drops the keys held and records that the folder is a key-file folder.
    /// The key file goes first: with the vault still in place a failure after
    /// it leaves a vault that says its key is missing, which can be started
    /// over again.
    fn set_aside_and_reset(&self, held: &mut Held, key_too: bool) -> Result<StartedOver, VaultError> {
        let key_kept_as = if key_too { set_aside(&self.dir, KEY_FILE_NAME, KEY_ASIDE, false)? } else { None };
        let vault_kept_as = set_aside(&self.dir, VAULT_FILE_NAME, VAULT_ASIDE, false)?;
        held.unlocked = None;
        held.keychain = KeychainHold::Nothing;
        marker::write(&self.dir, Mode::KeyFile)?;
        Ok(StartedOver { vault_kept_as, key_kept_as })
    }
}

#[cfg(test)]
include!("shared_start_over.tests.rs");
