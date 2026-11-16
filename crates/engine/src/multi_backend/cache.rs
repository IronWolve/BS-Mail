//! The latest of each thing the screen shows about an account, kept for every
//! member, so the screen can be told again when the user switches to it. The
//! events are stored already remapped, as the screen would have seen them.

use model::{ConnectionState, Event};

#[derive(Default)]
pub(super) struct MemberCache {
    folders: Option<Event>,
    changes: Option<Event>,
    signature: Option<Event>,
    connection: Option<Event>,
    quota: Option<Event>,
    sync: Option<Event>,
}

impl MemberCache {
    /// Remembers `event` when it is one of the kinds that are replayed.
    pub fn store(&mut self, event: &Event) {
        match event {
            Event::FolderList { .. } => self.folders = Some(event.clone()),
            Event::ChangesAllowed { .. } => self.changes = Some(event.clone()),
            Event::AccountSignature { .. } => self.signature = Some(event.clone()),
            Event::ConnectionState { .. } => self.connection = Some(event.clone()),
            Event::Quota { .. } => self.quota = Some(event.clone()),
            Event::SyncProgress { .. } => self.sync = Some(event.clone()),
            Event::AccountList(_)
            | Event::AccountUnread { .. }
            | Event::HostInfo { .. }
            | Event::EnvelopePage { .. }
            | Event::OlderEnvelopes { .. }
            | Event::BodyReady { .. }
            | Event::RawText { .. }
            | Event::HtmlPage { .. }
            | Event::AttachmentData { .. }
            | Event::FlagsChanged { .. }
            | Event::TagsChanged { .. }
            | Event::SearchResults { .. }
            | Event::MessageSent { .. }
            | Event::OpFailed { .. } => {}
        }
    }

    /// What the screen needs about the account, in the order it is applied:
    /// folders, whether changes are allowed, signature, connection, quota,
    /// sync status. Only the ones the member has said.
    pub fn replay(&self) -> Vec<Event> {
        [&self.folders, &self.changes, &self.signature, &self.connection, &self.quota, &self.sync]
            .into_iter()
            .flatten()
            .cloned()
            .collect()
    }

    /// The latest connection state the member reported; `Connecting` before
    /// it has reported any.
    pub fn connection_state(&self) -> ConnectionState {
        if let Some(Event::ConnectionState { state, .. }) = &self.connection {
            *state
        } else {
            ConnectionState::Connecting
        }
    }
}

#[cfg(test)]
include!("cache.tests.rs");
