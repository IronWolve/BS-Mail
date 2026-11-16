//! A stand-in for an account whose file could not be used. It tells the
//! screen once that the account is offline, that nothing can be sent from it,
//! and why, and answers every command
//! for it with the same reason, so the account still shows in the list.

use model::{AccountId, Command, ConnectionState, Event, MailBackend, RawPart};

use super::member::Member;

struct FailedBackend {
    id: AccountId,
    reason: String,
    started: bool,
}

impl FailedBackend {
    fn failure(&self, operation: &str) -> Event {
        Event::OpFailed {
            account: Some(self.id.clone()),
            operation: operation.to_owned(),
            reason: self.reason.clone(),
        }
    }
}

/// The operation name a failure for `command` carries, or `None` for a
/// command that is not about one account (the vault passphrase) and so has
/// nothing to fail for.
fn operation_name(command: &Command) -> Option<&'static str> {
    match command {
        Command::OpenFolder { .. } => Some("open folder"),
        Command::LoadMore { .. } => Some("load older mail"),
        Command::FetchBody { .. } => Some("fetch body"),
        Command::FetchRaw { part: RawPart::Headers, .. } => Some("view headers"),
        Command::FetchRaw { part: RawPart::Source, .. } => Some("view source"),
        Command::PrepareHtml { .. } => Some("open in browser"),
        Command::SetFlags { .. } => Some("set flags"),
        Command::Move { .. } => Some("move"),
        Command::Delete { .. } => Some("delete"),
        Command::SetTag { .. } => Some("tag"),
        Command::CreateFolder { .. } => Some("new folder"),
        Command::RenameFolder { .. } => Some("rename folder"),
        Command::DeleteFolder { .. } => Some("delete folder"),
        Command::MarkAllRead { .. } => Some("mark all read"),
        Command::EmptyFolder { .. } => Some("empty folder"),
        Command::Send { .. } => Some("send"),
        Command::Search { .. } => Some("search"),
        Command::SyncNow { .. } => Some("sync"),
        Command::SelectAccount { .. } => Some("account"),
        Command::Unlock { .. } => None,
    }
}

impl MailBackend for FailedBackend {
    fn handle(&mut self, command: Command) -> Vec<Event> {
        operation_name(&command)
            .map(|operation| vec![self.failure(operation)])
            .unwrap_or_default()
    }

    fn poll_events(&mut self) -> Vec<Event> {
        if self.started {
            return Vec::new();
        }
        self.started = true;
        vec![
            Event::ConnectionState {
                account: self.id.clone(),
                state: ConnectionState::Offline,
            },
            // Nothing can be sent from an account that is not running, so the
            // composer does not offer it.
            Event::ChangesAllowed {
                account: self.id.clone(),
                allowed: false,
            },
            self.failure("account"),
        ]
    }
}

impl Member {
    /// A member for an account that could not be started. At start it reports
    /// `ConnectionState::Offline`, that changes are not allowed and one
    /// failure named `account` with `reason`; every command for it is then answered with a failure
    /// carrying the same reason.
    pub fn failed(
        id: AccountId,
        name: impl Into<String>,
        address: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        let backend = FailedBackend {
            id: id.clone(),
            reason: reason.into(),
            started: false,
        };
        Self::new(id, name, address, Box::new(backend))
    }
}

#[cfg(test)]
include!("failed_member.tests.rs");
