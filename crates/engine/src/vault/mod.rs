//! The encrypted vault the program keeps secrets in: one file,
//! `<data dir>/secrets/vault`, holding named secret texts such as
//! `accounts.<id>.password`. Every change in the program goes through the one
//! shared handle on the folder (`shared.rs`). This file only declares and
//! re-exports the modules; the format is described in `format.rs` and in
//! section 9.1 of the plan.

mod crypto;
mod entries;
mod error;
mod file_io;
mod format;
mod inject;
mod kdf;
mod key_file;
mod key_source;
mod keychain;
mod names;
mod shared;
mod shared_keychain;
mod store;
mod switch;
mod switch_keychain;
mod view;

pub use error::VaultError;
pub use format::{MAX_VAULT_BYTES, Mode};
pub use key_source::{KEY_FILE_NAME, KeySource, KeychainKey, Unlocked};
pub use keychain::{KEY_ID_LEN, KeyId, KeychainFailure, KeychainStore, StoredKey, keychain_built, set_keychain_service};
#[cfg(any(test, feature = "test-support"))]
pub use keychain::{FakeKeychain, Op};
pub use names::{
    TokenNames, account_prefix, attempt_name, client_secret_name, earlier_token_names, password_name,
    provider_client_secret_name, smtp_password_name, token_names,
};
pub use shared::{Protection, SharedVault};
pub use store::{SECRETS_FOLDER_NAME, VAULT_FILE_NAME, Vault};
pub use switch::Switched;
pub use view::VaultView;
