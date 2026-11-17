//! The system keychain as a place for the vault's key: the store it is kept
//! in (a trait with three implementations: the real one over the operating
//! system's own store, a fake in memory for tests, and one that says there is
//! no keychain), the way every call to it is made (on a thread of its own,
//! with a time limit) and the names things are kept under. This file only
//! declares and re-exports the modules; section 9.1 of the plan describes the
//! mode.

mod call;
mod default;
mod failure;
mod hex;
mod key_id;
mod store;
mod stored_key;
mod unavailable;
mod wrapper;

#[cfg(any(test, feature = "test-support"))]
mod fake;
#[cfg(any(windows, target_os = "linux"))]
mod real;

pub use default::{keychain_built, set_keychain_service};
pub(super) use default::default_store;
pub use failure::KeychainFailure;
pub use key_id::{KEY_ID_LEN, KeyId};
pub use store::KeychainStore;
pub use stored_key::StoredKey;
pub(super) use wrapper::Keychain;

#[cfg(any(test, feature = "test-support"))]
pub use fake::{FakeKeychain, Op};
