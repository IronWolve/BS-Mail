//! The IMAP side of the real backend: one signed-in, read-only session and
//! the pure helpers that turn server data into model types. This file only
//! declares and re-exports the modules.

mod body_structure;
mod capabilities;
mod connect;
mod envelope_map;
mod error;
mod fetch_body;
mod fetch_envelopes;
mod flag_map;
mod folder_list;
mod folder_roles;
mod folder_tree;
mod header_text;
mod login;
mod mutf7;
mod quota;
mod raw_folder;
mod select;
mod session;
mod stream;
mod timeouts;
mod tls_config;

pub use envelope_map::build_envelope;
pub use error::{ConnectError, SessionError};
pub use fetch_envelopes::fetched_row;
pub use folder_tree::build_tree;
pub use session::ImapSession;
pub use timeouts::Timeouts;
