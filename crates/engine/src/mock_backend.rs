//! The mock mail backend: serves the fixture and answers commands the way the
//! real engine will, so the UI can be built and tested without a server.

use std::path::Path;
use std::str::FromStr;

use model::{
    AccountId, Body, BodyRole, Command, ConnectionState, Envelope, Event, FlagChange, FolderId,
    MailBackend, MailDate, MessageId, RequestId, StyledLine, StyledSpan, SyncStatus,
};

use crate::mock::mutate::{MoveOutcome, apply_flags, delete_messages, move_messages};
use crate::mock::search;
use crate::mock::{Fixture, FixtureError};

/// What a message with no fixture body shows when it is opened.
const PLACEHOLDER_BODY: &str = "(The mock fixture holds no body for this message.)";

/// A backend that serves a fixture and keeps changes in memory only.
///
/// `MockBackend::from_str` (through `FromStr`) and [`MockBackend::from_file`]
/// build one. The first `poll_events` call delivers [`startup_events`]; after
/// that the only events are the answers to commands.
///
/// [`startup_events`]: MockBackend::startup_events
#[derive(Debug, Clone)]
pub struct MockBackend {
    data: Fixture,
    started: bool,
}

impl MockBackend {
    /// Loads the fixture file at `path`.
    pub fn from_file(path: &Path) -> Result<Self, FixtureError> {
        Fixture::from_file(path).map(Self::from_fixture)
    }

    /// Serves already-converted fixture data.
    pub fn from_fixture(data: Fixture) -> Self {
        Self {
            data,
            started: false,
        }
    }

    /// The data being served, including any changes made by commands.
    pub fn fixture(&self) -> &Fixture {
        &self.data
    }

    /// The fixture's "now", for the UI's Today and Yesterday.
    pub fn now(&self) -> MailDate {
        self.data.now
    }

    /// Everything the UI needs for its first frame, in the order it should
    /// be applied: host, accounts, folders, connection, quota, sync status,
    /// the open folder's first page, then the open message's body.
    pub fn startup_events(&self) -> Vec<Event> {
        let account = self.data.account.clone();
        let mut events = vec![
            Event::HostInfo {
                host_name: self.data.host_name.clone(),
                system: self.data.system.clone(),
            },
            Event::AccountList(self.data.accounts.clone()),
            self.folder_list(),
            Event::ConnectionState {
                account: account.clone(),
                state: self.selected_connection(),
            },
            Event::Quota {
                account: account.clone(),
                quota: self.data.status.quota,
            },
            Event::SyncProgress {
                account,
                status: self.data.status.clone(),
            },
        ];
        if let Some(folder) = self.data.open_folder() {
            events.push(self.envelope_page(folder));
            events.push(Event::BodyReady {
                message: self.data.open_message,
                body: self.data.open_body.clone(),
            });
        }
        events
    }

    fn selected_connection(&self) -> ConnectionState {
        self.data
            .accounts
            .iter()
            .find(|account| account.selected)
            .map_or(ConnectionState::Offline, |account| account.state)
    }

    fn folder_list(&self) -> Event {
        Event::FolderList {
            account: self.data.account.clone(),
            folders: self.data.folders.clone(),
        }
    }

    /// The newest page of a folder: every fixture message filed there, newest
    /// first. The total is the folder's own count, which is larger than the
    /// page when the fixture holds only a sample; a folder with no messages
    /// is empty.
    fn envelope_page(&self, folder: FolderId) -> Event {
        let mut envelopes: Vec<Envelope> = self
            .data
            .messages
            .iter()
            .filter(|envelope| envelope.folder == folder)
            .cloned()
            .collect();
        envelopes.sort_by(|a, b| b.date.unix.cmp(&a.date.unix));
        let listed = u32::try_from(envelopes.len()).unwrap_or(u32::MAX);
        let total = match self.data.folders.iter().find(|entry| entry.id == folder) {
            Some(entry) if listed > 0 => entry.total.max(listed),
            _ => 0,
        };
        Event::EnvelopePage {
            folder,
            offset: 0,
            total,
            envelopes,
        }
    }

    fn failure(&self, operation: &str, reason: String) -> Event {
        Event::OpFailed {
            account: Some(self.data.account.clone()),
            operation: operation.to_owned(),
            reason,
        }
    }

    fn unknown_messages(&self, operation: &str, ids: &[MessageId]) -> Option<Event> {
        if ids.is_empty() {
            return None;
        }
        let list: Vec<String> = ids.iter().map(|id| id.0.to_string()).collect();
        Some(self.failure(
            operation,
            format!("unknown message id: {}", list.join(", ")),
        ))
    }

    fn knows_folder(&self, folder: FolderId) -> bool {
        self.data.folders.iter().any(|entry| entry.id == folder)
    }

    fn open_folder(&self, folder: FolderId) -> Vec<Event> {
        if self.knows_folder(folder) {
            vec![self.envelope_page(folder)]
        } else {
            vec![self.failure("open folder", format!("unknown folder id: {}", folder.0))]
        }
    }

    fn fetch_body(&self, message: MessageId) -> Vec<Event> {
        if !self
            .data
            .messages
            .iter()
            .any(|envelope| envelope.id == message)
        {
            return vec![self.failure("fetch body", format!("unknown message id: {}", message.0))];
        }
        let body = if message == self.data.open_message {
            self.data.open_body.clone()
        } else {
            placeholder_body()
        };
        vec![Event::BodyReady { message, body }]
    }

    fn set_flags(&mut self, ids: &[MessageId], change: FlagChange) -> Vec<Event> {
        let outcome = apply_flags(&mut self.data, ids, change);
        let mut events = Vec::new();
        if !outcome.updates.is_empty() {
            events.push(Event::FlagsChanged {
                updates: outcome.updates,
            });
        }
        if outcome.counts_moved {
            events.push(self.folder_list());
        }
        events.extend(self.unknown_messages("set flags", &outcome.unknown));
        events
    }

    fn move_to(&mut self, ids: &[MessageId], to: FolderId) -> Vec<Event> {
        if !self.knows_folder(to) {
            return vec![self.failure("move", format!("unknown folder id: {}", to.0))];
        }
        let outcome = move_messages(&mut self.data, ids, to);
        self.after_relocation("move", outcome)
    }

    fn delete(&mut self, ids: &[MessageId]) -> Vec<Event> {
        let outcome = delete_messages(&mut self.data, ids);
        self.after_relocation("delete", outcome)
    }

    /// The updated folder counts, then a fresh page for each folder that
    /// lost or gained messages.
    fn after_relocation(&self, operation: &str, outcome: MoveOutcome) -> Vec<Event> {
        let mut events = Vec::new();
        if !outcome.touched.is_empty() {
            events.push(self.folder_list());
            events.extend(
                outcome
                    .touched
                    .iter()
                    .map(|&folder| self.envelope_page(folder)),
            );
        }
        events.extend(self.unknown_messages(operation, &outcome.unknown));
        events
    }

    fn search(&self, request: RequestId, account: &AccountId, query: &str) -> Vec<Event> {
        if !self.data.accounts.iter().any(|entry| &entry.id == account) {
            return vec![self.failure("search", format!("unknown account: {}", account.0))];
        }
        let envelopes: Vec<Envelope> = if *account == self.data.account {
            self.data
                .messages
                .iter()
                .filter(|envelope| search::matches(envelope, query))
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        vec![Event::SearchResults {
            request,
            total: u32::try_from(envelopes.len()).unwrap_or(u32::MAX),
            envelopes,
        }]
    }

    /// A sync that finishes at once with the fixture's status; accounts the
    /// fixture holds no data for report an empty status.
    fn sync_now(&self, account: &AccountId) -> Vec<Event> {
        if !self.data.accounts.iter().any(|entry| &entry.id == account) {
            return vec![self.failure("sync", format!("unknown account: {}", account.0))];
        }
        let status = if *account == self.data.account {
            self.data.status.clone()
        } else {
            SyncStatus::default()
        };
        vec![
            Event::Quota {
                account: account.clone(),
                quota: status.quota,
            },
            Event::SyncProgress {
                account: account.clone(),
                status,
            },
        ]
    }
}

fn placeholder_body() -> Body {
    Body {
        lines: vec![StyledLine {
            spans: vec![StyledSpan {
                text: PLACEHOLDER_BODY.to_owned(),
                role: BodyRole::Text,
            }],
        }],
        attachments: Vec::new(),
    }
}

impl FromStr for MockBackend {
    type Err = FixtureError;

    /// Builds a backend from fixture text; errors name the file as `<inline>`.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Fixture::from_text(text).map(Self::from_fixture)
    }
}

impl MailBackend for MockBackend {
    /// Answers at once. A command the mock has nothing to say about (`Send`,
    /// `Unlock`) gets no events.
    fn handle(&mut self, command: Command) -> Vec<Event> {
        match command {
            Command::OpenFolder { folder } => self.open_folder(folder),
            Command::FetchBody { message } => self.fetch_body(message),
            Command::SetFlags { messages, change } => self.set_flags(&messages, change),
            Command::Move { messages, to } => self.move_to(&messages, to),
            Command::Delete { messages } => self.delete(&messages),
            Command::Search {
                request,
                account,
                query,
            } => self.search(request, &account, &query),
            Command::SyncNow { account } => self.sync_now(&account),
            Command::Send { .. } | Command::Unlock { .. } => Vec::new(),
        }
    }

    /// The first call returns the start-up events; later calls return none.
    fn poll_events(&mut self) -> Vec<Event> {
        if self.started {
            return Vec::new();
        }
        self.started = true;
        self.startup_events()
    }
}

#[cfg(test)]
include!("mock_backend.tests.rs");
