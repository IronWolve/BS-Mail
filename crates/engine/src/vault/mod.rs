//! The encrypted vault the program keeps secrets in: one file,
//! `<data dir>/secrets/vault`, holding named secret texts such as
//! `accounts.<id>.password`. This file only declares and re-exports the
//! modules; the format is described in `format.rs` and in section 9.1 of the
//! plan.

mod crypto;
mod entries;
mod error;
mod file_io;
mod format;
mod key_file;
mod key_source;
mod names;
mod store;
mod view;

pub use error::VaultError;
pub use format::{MAX_VAULT_BYTES, Mode};
pub use key_source::{KEY_FILE_NAME, KeySource};
pub use names::{account_prefix, password_name, smtp_password_name};
pub use store::{VAULT_FILE_NAME, Vault};
pub use view::VaultView;
