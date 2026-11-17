//! Which store the program uses when nothing else was given: the operating
//! system's own keychain (Linux and Windows), under the service name the
//! program was started with, or, on any other system and when no name was
//! given, a store that says there is no keychain. The name is the program's
//! own (the app passes it from its identity): the engine never spells it.

use std::sync::{Arc, OnceLock};

use super::store::KeychainStore;
use super::unavailable::Unavailable;

static SERVICE: OnceLock<String> = OnceLock::new();

/// Tells the engine the service name its keychain entries are kept under:
/// the program's own short name. Called once, at start; a second call changes
/// nothing.
pub fn set_keychain_service(name: &str) {
    let _ = SERVICE.set(name.to_string());
}

/// Whether this build has code for the system's own keychain: Linux and
/// Windows. Elsewhere (macOS, which this program is not tested on) there is
/// none, and the Options screen says so in other words than "none was found".
pub fn keychain_built() -> bool {
    cfg!(any(windows, target_os = "linux"))
}

/// The store to use when a vault folder's handle was given none.
pub(in crate::vault) fn default_store() -> Arc<dyn KeychainStore> {
    static DEFAULT: OnceLock<Arc<dyn KeychainStore>> = OnceLock::new();
    match SERVICE.get() {
        Some(service) => Arc::clone(DEFAULT.get_or_init(|| system_store(service))),
        None => Arc::new(Unavailable),
    }
}

#[cfg(any(windows, target_os = "linux"))]
fn system_store(service: &str) -> Arc<dyn KeychainStore> {
    Arc::new(super::real::SystemKeychain::new(service))
}

/// macOS has a keychain too, but this program is not tested against it, so it
/// is not offered there (and no code for it is built).
#[cfg(not(any(windows, target_os = "linux")))]
fn system_store(_service: &str) -> Arc<dyn KeychainStore> {
    Arc::new(Unavailable)
}

#[cfg(test)]
include!("default.tests.rs");
