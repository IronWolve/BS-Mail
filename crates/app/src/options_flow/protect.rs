//! Changing how the saved passwords are protected, as the Options screen asks:
//! to a passphrase, to another passphrase, to a key file, to the system keychain,
//! and looking for a keychain at all. Each change is one change of the vault through
//! the process's one handle on it (`engine::vault::SharedVault`), atomic as
//! `engine/src/vault/switch.rs` says: on an error the vault and its key are
//! exactly as they were, and the line at the bottom of the Options screen says
//! so. The Data page is told the way now in force only when the change went
//! through. A passphrase is read here from a `Secret` and never put in a sentence.
//!
//! A change that is to or from the keychain, and the look for a keychain, can
//! wait on the system, so they run on a thread of their own (`Background`) and
//! the line says it is working; the others take a moment and are done at once.

use engine::vault::{SharedVault, Switched, VaultError};
use model::Secret;
use ui::AppView;
use ui::options::{KeychainState, Protection};

use crate::accounts::Live;

use super::answer::live_or_problem;
use super::background::{Apply, Background};

/// Said when a second change is asked for while one is still working.
const BUSY: &str = "Another change to the passwords is still being made; try again in a moment.";

/// The vault of this run's accounts.
fn vault_of(live: &Live) -> std::sync::Arc<SharedVault> {
    SharedVault::for_dir(&live.place.secrets_folder())
}

/// Whether the passwords are protected by the system keychain now.
fn in_the_keychain(view: &AppView) -> bool {
    view.protection() == Some(Protection::Keychain)
}

/// Protects the passwords with `passphrase`, in place of a key file or the keychain.
pub(super) fn use_passphrase(view: &mut AppView, live: Option<&Live>, jobs: &mut Background, passphrase: &Secret) {
    let Some(live) = live_or_problem(view, live) else {
        return;
    };
    let vault = vault_of(live);
    let said = "Passwords are now protected by a passphrase.";
    if in_the_keychain(view) {
        let passphrase = passphrase.clone();
        let work = move || finishing(vault.switch_to_passphrase(&passphrase), Protection::Passphrase, said);
        return run_apart(view, jobs, "Moving the passwords out of the system keychain...", work);
    }
    done(view, vault.switch_to_passphrase(passphrase), Protection::Passphrase, said);
}

/// Uses `passphrase` in place of the one in force.
pub(super) fn change_passphrase(view: &mut AppView, live: Option<&Live>, passphrase: &Secret) {
    let Some(live) = live_or_problem(view, live) else {
        return;
    };
    let result = vault_of(live).change_passphrase(passphrase);
    done(view, result, Protection::Passphrase, "The passphrase is changed.");
}

/// Protects the passwords with a key file again, in place of a passphrase or the keychain.
pub(super) fn use_key_file(view: &mut AppView, live: Option<&Live>, jobs: &mut Background) {
    let Some(live) = live_or_problem(view, live) else {
        return;
    };
    let vault = vault_of(live);
    let said = "Passwords are now protected by a key file.";
    if in_the_keychain(view) {
        let work = move || finishing(vault.switch_to_key_file(), Protection::KeyFile, said);
        return run_apart(view, jobs, "Moving the passwords out of the system keychain...", work);
    }
    done(view, vault.switch_to_key_file(), Protection::KeyFile, said);
}

/// Keeps the key of the passwords in the system keychain.
pub(super) fn use_keychain(view: &mut AppView, live: Option<&Live>, jobs: &mut Background) {
    let Some(live) = live_or_problem(view, live) else {
        return;
    };
    let vault = vault_of(live);
    let work = move || finishing(vault.switch_to_keychain(), Protection::Keychain, "Passwords are now protected by the system keychain.");
    run_apart(view, jobs, "Moving the key into the system keychain...", work);
}

/// Looks whether this computer has a keychain the program can use, and tells
/// the Data page. The look can wait on the system, so it is made apart.
pub(super) fn check_keychain(view: &mut AppView, live: Option<&Live>, jobs: &mut Background) {
    if !engine::vault::keychain_built() {
        return view.set_keychain(KeychainState::NotBuilt);
    }
    let Some(live) = live else {
        return view.set_keychain(KeychainState::Unavailable);
    };
    let vault = vault_of(live);
    let started = jobs.start(move || {
        let found = vault.keychain_probe().is_ok();
        let apply: Apply = Box::new(move |view| view.set_keychain(if found { KeychainState::Available } else { KeychainState::Unavailable }));
        apply
    });
    if !started {
        // Busy with a change: the page asks again the next time it is opened.
        view.set_keychain(KeychainState::Unknown);
    }
}

/// Starts `work` apart and says it is working, or says why it was not started.
fn run_apart(view: &mut AppView, jobs: &mut Background, working: &str, work: impl FnOnce() -> Apply + Send + 'static) {
    if jobs.start(work) {
        view.options_info(working);
    } else {
        view.options_problem(BUSY);
    }
}

/// What a finished change does to the screen, made where the change was.
fn finishing(result: Result<Switched, VaultError>, now: Protection, said: &'static str) -> Apply {
    Box::new(move |view| done(view, result, now, said))
}

/// Tells the screen how the change went.
fn done(view: &mut AppView, result: Result<Switched, VaultError>, now: Protection, said: &str) {
    match result {
        Ok(switched) => {
            view.set_protection(now);
            match (switched.key_file_left, switched.keychain_left) {
                (None, None) => view.options_info(said),
                (Some(why), _) => view.options_problem(&format!(
                    "Done, but the old key file could not be removed ({why}): delete it. It opens nothing now."
                )),
                (None, Some(why)) => view.options_problem(&format!(
                    "Done, but the old key is still in the system keychain ({why}); it opens nothing now."
                )),
            }
        }
        Err(error) => view.options_problem(&format!("Nothing was changed: {error}.")),
    }
}

#[cfg(test)]
include!("protect.tests.rs");
