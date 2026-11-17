//! Shared plain types: mail data, commands, events, config tables and the
//! theme file schema. No IO and no drawing.

pub mod address;
pub mod attachments;
pub mod backend;
pub mod body;
pub mod command;
pub mod config;
pub mod envelope;
pub mod event;
pub mod flags;
pub mod folder;
pub mod file_bytes;
pub mod folder_rules;
pub mod ids;
pub mod media_types;
pub mod names;
pub mod poll;
pub mod sync_status;
pub mod theme_schema;
pub mod thread_headers;

pub use address::Address;
pub use attachments::{MAX_FETCH_BYTES, MAX_SEND_BYTES, OutgoingAttachment, TEXT_RESERVE, max_raw_len, sendable_name, wire_size};
pub use backend::MailBackend;
pub use body::{AttachmentInfo, Body, BodyRole, StyledLine, StyledSpan};
pub use command::{Command, OutgoingMessage, RawPart, Secret, wipe_bytes, wipe_text};
pub use envelope::{Envelope, MailDate};
pub use event::Event;
pub use file_bytes::FileBytes;
pub use flags::{FlagChange, Flags};
pub use folder::{Account, Folder, FolderRole};
pub use folder_rules::{holds_folders, not_emptiable, protected, saved_search_refusal};
pub use ids::{AccountId, FolderId, MessageId, RequestId, ThreadId, Uid};
pub use poll::{DEFAULT_POLL_SECONDS, MIN_POLL_SECONDS};
pub use names::{BOOKKEEPING_KEYWORDS, MAX_FOLDER_NAME, MAX_TAG, check_folder_name, check_tag};
pub use sync_status::{ConnectionState, Quota, SyncState, SyncStatus};
pub use thread_headers::{ThreadHeaders, normal_id};
