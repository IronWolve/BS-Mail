//! The commands this milestone does not carry out: the backend only reads.

use model::Command;

/// The reason shown for a command that would change the server.
pub const READ_ONLY_REASON: &str = "not available yet (read-only)";

/// The operation name for a command that is not available, or `None` for a
/// command that is.
pub fn unavailable_operation(command: &Command) -> Option<&'static str> {
    match command {
        Command::SetFlags { .. } => Some("set flags"),
        Command::Move { .. } => Some("move"),
        Command::Delete { .. } => Some("delete"),
        Command::Send { .. } => Some("send"),
        Command::Search { .. } => Some("search"),
        Command::Unlock { .. } => Some("unlock"),
        Command::OpenFolder { .. } | Command::FetchBody { .. } | Command::SyncNow { .. } => None,
    }
}

#[cfg(test)]
include!("unsupported.tests.rs");
