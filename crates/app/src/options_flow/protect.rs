//! Changing how the saved passwords are protected, as the Options screen asks:
//! to a passphrase, to another passphrase, or back to a key file. Each is one
//! change of the vault through the process's one handle on it
//! (`engine::vault::SharedVault`), atomic as `engine/src/vault/switch.rs`
//! says: on an error the vault and its key are exactly as they were, and the
//! line at the bottom of the Options screen says so. The Data page is told the
//! way now in force only when the change went through. A passphrase is read
//! here from a `Secret` and never put in a sentence.

use engine::vault::{SharedVault, Switched};
use model::Secret;
use ui::AppView;
use ui::options::Protection;

use crate::accounts::Live;

use super::answer::live_or_problem;

/// The vault of this run's accounts.
fn vault_of(live: &Live) -> std::sync::Arc<SharedVault> {
    SharedVault::for_dir(&live.place.secrets_folder())
}

/// Protects the passwords with `passphrase`, in place of a key file.
pub(super) fn use_passphrase(view: &mut AppView, live: Option<&Live>, passphrase: &Secret) {
    let Some(live) = live_or_problem(view, live) else {
        return;
    };
    let result = vault_of(live).switch_to_passphrase(passphrase);
    done(view, result, Protection::Passphrase, "Passwords are now protected by a passphrase.");
}

/// Uses `passphrase` in place of the one in force.
pub(super) fn change_passphrase(view: &mut AppView, live: Option<&Live>, passphrase: &Secret) {
    let Some(live) = live_or_problem(view, live) else {
        return;
    };
    let result = vault_of(live).change_passphrase(passphrase);
    done(view, result, Protection::Passphrase, "The passphrase is changed.");
}

/// Protects the passwords with a key file again.
pub(super) fn use_key_file(view: &mut AppView, live: Option<&Live>) {
    let Some(live) = live_or_problem(view, live) else {
        return;
    };
    let result = vault_of(live).switch_to_key_file().map(|()| Switched::default());
    done(view, result, Protection::KeyFile, "Passwords are now protected by a key file.");
}

/// Tells the screen how the change went.
fn done(view: &mut AppView, result: Result<Switched, engine::vault::VaultError>, now: Protection, said: &str) {
    match result {
        Ok(switched) => {
            view.set_protection(now);
            match switched.key_file_left {
                None => view.options_info(said),
                Some(why) => view.options_problem(&format!(
                    "Done, but the old key file could not be removed ({why}): delete it. It opens nothing now."
                )),
            }
        }
        Err(error) => view.options_problem(&format!("Nothing was changed: {error}.")),
    }
}

#[cfg(test)]
include!("protect.tests.rs");
