//! The commands the backend does not carry out. Unlock is never available
//! yet; changing mail (flags, move, delete) and sending mail are
//! refused while the account is read-only and served when it is not.

use model::Command;

/// The reason shown for a command that would change the server on an account
/// that is read-only.
pub const READ_ONLY_REASON: &str = "not available yet (read-only)";

/// The reason shown for a send on an account that is read-only.
pub const SEND_READ_ONLY_REASON: &str = "this account is read-only";

/// The reason shown for a command no account can use yet.
pub const NOT_YET_REASON: &str = "not available yet";

/// The reason shown for a change asked for while the connection is down. The
/// command is not kept and not sent when the connection comes back.
pub const OFFLINE_CHANGE_REASON: &str = "offline: not connected to the server; nothing was changed";

/// The operation name for a command that changes mail on the server.
pub fn change_operation(command: &Command) -> Option<&'static str> {
    match command {
        Command::SetFlags { .. } => Some("set flags"),
        Command::Move { .. } => Some("move"),
        Command::Delete { .. } => Some("delete"),
        _ => None,
    }
}

/// The operation name for a command that is not available, or `None` for a
/// command that is. `writes` is whether the account may change the server.
pub fn unavailable_operation(command: &Command, writes: bool) -> Option<&'static str> {
    match command {
        Command::Send { .. } if !writes => Some("send"),
        Command::Unlock { .. } => Some("unlock"),
        other if !writes => change_operation(other),
        _ => None,
    }
}

/// Why a command from `unavailable_operation` is refused.
pub const fn unavailable_reason(command: &Command, writes: bool) -> &'static str {
    match command {
        Command::Send { .. } if !writes => SEND_READ_ONLY_REASON,
        _ if writes => NOT_YET_REASON,
        _ => READ_ONLY_REASON,
    }
}

#[cfg(test)]
include!("unsupported.tests.rs");
