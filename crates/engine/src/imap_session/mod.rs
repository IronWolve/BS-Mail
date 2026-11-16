//! The IMAP side of the real backend: one signed-in session, the few
//! operations that can change a mailbox when the account allows it, and the
//! pure helpers that turn server data into model types. This file only
//! declares and re-exports the modules. The TLS settings and the stream type
//! are shared with the sending side, so both judge certificates the same way.

mod body_structure;
mod capabilities;
mod connect;
mod envelope_map;
mod error;
mod fetch_body;
mod fetch_envelopes;
mod fetch_headers;
mod flag_map;
mod folder_list;
mod folder_roles;
mod folder_tree;
mod header_text;
mod login;
mod login_modes;
mod mailbox_change;
mod move_copy;
mod mutf7;
mod sent_check;
mod quota;
mod raw_folder;
mod search_exchange;
mod select;
mod session;
mod store_flags;
mod status_poll;
mod stream;
mod timeouts;
mod tls_config;
mod uid_set;
mod write_command;
mod write_exchange;
mod write_select;

pub use capabilities::ServerCaps;
pub use envelope_map::build_envelope;
pub use error::{ConnectError, SessionError, classify_tls};
#[cfg(test)]
pub use error::server_words;
pub use fetch_envelopes::fetched_row;
pub use folder_roles::Unmatched as UnmatchedRole;
pub use folder_tree::build_tree;
pub use mutf7::{decode as decode_mailbox_name, encode as encode_mailbox_name};
pub use search_exchange::{LITERAL_REFUSAL, SENDS_LITERALS};
pub use session::ImapSession;
pub use status_poll::StatusCounts;
pub use stream::MailStream;
pub use timeouts::Timeouts;
pub use tls_config::client_config;
pub use uid_set::{MAX_PER_COMMAND, UidSet};
pub use write_command::{Keyword, SentFolder, StoreFlag, StoreMode};

#[cfg(test)]
include!("test_server.tests.rs");
