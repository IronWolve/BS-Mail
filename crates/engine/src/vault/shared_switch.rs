//! The shared handle's switches between the ways of protecting the vault that
//! need no keychain call: to a passphrase, to another passphrase and to a key
//! file (the moves to the keychain are in `shared_keychain.rs`). Each is one
//! change under the handle's lock and the folder's lock, atomic as `switch.rs`
//! says. The key of a keychain vault that was left is removed after the lock
//! is let go, and not at all when the new vault could not be flushed to disk.

use model::Secret;

use super::error::VaultError;
use super::format::Mode;
use super::shared::{KeychainHold, SharedVault};
use super::store::Vault;
use super::switch::{self, Switched};

impl SharedVault {
    /// Protects the vault with `passphrase` from now on: a key-file vault (or
    /// none yet) becomes a passphrase vault, a passphrase vault that is open
    /// gets a new passphrase and a new key-derivation salt. Atomic: see
    /// `switch.rs`. On an error the old vault and key are as they were.
    pub fn switch_to_passphrase(&self, passphrase: &Secret) -> Result<Switched, VaultError> {
        let (mut switched, old_key) = {
            let mut held = self.hold();
            let (_lock, source) = self.begin(&mut held, true)?;
            let vault = Vault::open_or_create(&self.dir, source?)?;
            let old_key = vault.keychain_id();
            let (unlocked, switched) = switch::to_passphrase(&vault, passphrase)?;
            held.unlocked = Some(unlocked);
            held.keychain = KeychainHold::Nothing;
            (switched, old_key)
        };
        // The new vault is in place; only now does the old keychain key go, and with the lock let go.
        self.remove_old_keychain_key(old_key, &mut switched);
        Ok(switched)
    }

    /// As `switch_to_passphrase`, for a vault that is a passphrase vault
    /// already: `WrongMode` for any other.
    pub fn change_passphrase(&self, passphrase: &Secret) -> Result<Switched, VaultError> {
        if self.protection().mode != Mode::Passphrase {
            return Err(VaultError::WrongMode);
        }
        self.switch_to_passphrase(passphrase)
    }

    /// Protects the vault with a new random key file instead of a passphrase
    /// or the system keychain; the vault must be open (`Locked`, or the
    /// keychain's own failure, otherwise). A vault that is a key-file vault
    /// already is left as it is. The key held is dropped, and a keychain key
    /// is removed from the keychain once the new vault is in place.
    pub fn switch_to_key_file(&self) -> Result<Switched, VaultError> {
        let (mut switched, old_key) = {
            let mut held = self.hold();
            if self.peek().is_none_or(|header| header.mode == Mode::KeyFile) {
                return Ok(Switched::default());
            }
            let (_lock, source) = self.begin(&mut held, true)?;
            let vault = Vault::open_or_create(&self.dir, source?)?;
            let old_key = vault.keychain_id();
            let not_synced = switch::to_key_file(&vault)?;
            held.unlocked = None;
            held.keychain = KeychainHold::Nothing;
            (Switched { not_synced, ..Switched::default() }, old_key)
        };
        self.remove_old_keychain_key(old_key, &mut switched);
        Ok(switched)
    }
}
