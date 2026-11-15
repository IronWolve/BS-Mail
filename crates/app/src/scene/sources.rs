//! One scene for each kind of source.

use std::path::{Path, PathBuf};

use engine::{AccountConfig, Member, MockBackend, MultiBackend, RealBackend};
use model::AccountId;
use model::config::WindowConfig;
use ui::AppView;

use crate::accounts::{Live, Place, Scan, members_of, reason_of};
use crate::errors::AppError;
use crate::options_flow::Settings;

use super::state::Scene;

/// What every scene needs from the settings.
pub(super) type Base = (host::Rgb, host::Rgb);

/// Mock mail from a fixture file: a plain mock backend, no accounts.
pub(super) fn mock(mut view: AppView, base: Base, path: &Path) -> Result<Scene, AppError> {
    let backend = MockBackend::from_file(path)
        .map_err(|error| AppError::Fixture(path.to_path_buf(), error.to_string()))?;
    view.set_now(backend.now());
    Ok(Scene {
        view,
        messages: backend.fixture().messages.len(),
        fixed_now: Some(backend.now()),
        font: (12, 1.0),
        window: WindowConfig::default(),
        backend: Box::new(backend),
        source: path.to_path_buf(),
        label: "mock data",
        real: false,
        accounts: None,
        live: None,
        base,
        settings: Settings::default(),
        lock: None,
    })
}

/// `--account <file>`: that one account, as the only member. A file that
/// cannot be used stops the run, as it always has.
pub(super) fn single(view: AppView, base: Base, file: &Path) -> Result<Scene, AppError> {
    let config = AccountConfig::from_file(file)
        .map_err(|error| AppError::Account(file.to_path_buf(), reason_of(&error)))?;
    let id = AccountId(config.id.clone());
    let member = Member::new(id, config.name.clone(), config.address.clone(), Box::new(RealBackend::start(config)));
    Ok(real(view, base, vec![member], Place::File(file.to_path_buf()), (file.to_path_buf(), "account")))
}

/// The accounts found in the accounts folder, each its own member. One that
/// cannot be used is a member that says so.
pub(super) fn many(view: AppView, base: Base, root: &Path, scan: &Scan) -> Scene {
    let members = members_of(&scan.found);
    real(view, base, members, Place::Root(root.to_path_buf()), (root.to_path_buf(), "accounts in"))
}

/// Nothing is running yet: the form will make the first account, saved in
/// `place`.
pub(super) fn empty(view: AppView, base: Base, place: Place, source: PathBuf) -> Scene {
    real(view, base, Vec::new(), place, (source, "saves to"))
}

fn real(mut view: AppView, base: Base, members: Vec<Member>, place: Place, source: (PathBuf, &'static str)) -> Scene {
    let count = members.len();
    let multi = MultiBackend::new(members);
    let live = Live::new(place, multi.members());
    view.set_saved_clients(live.clients().load());
    Scene {
        view,
        messages: 0,
        fixed_now: None,
        font: (12, 1.0),
        window: WindowConfig::default(),
        backend: Box::new(multi),
        source: source.0,
        label: source.1,
        real: count > 0,
        accounts: Some(count),
        live: Some(live),
        base,
        settings: Settings::default(),
        lock: None,
    }
}

#[cfg(test)]
include!("sources.tests.rs");
