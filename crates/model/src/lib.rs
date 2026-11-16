//! Shared plain types: mail data, commands, events, config tables and the
//! theme file schema. No IO and no drawing.

pub mod address;
pub mod backend;
pub mod body;
pub mod command;
pub mod config;
pub mod envelope;
pub mod event;
pub mod flags;
pub mod folder;
pub mod ids;
pub mod sync_status;
pub mod theme_schema;

pub use address::Address;
pub use backend::MailBackend;
pub use body::{AttachmentInfo, Body, BodyRole, StyledLine, StyledSpan};
pub use command::{Command, OutgoingMessage, RawPart, Secret, wipe_text};
pub use envelope::{Envelope, MailDate};
pub use event::Event;
pub use flags::{FlagChange, Flags};
pub use folder::{Account, Folder, FolderRole};
pub use ids::{AccountId, FolderId, MessageId, RequestId, ThreadId, Uid};
pub use sync_status::{ConnectionState, Quota, SyncState, SyncStatus};
