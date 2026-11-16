//! The mock mail backend: serves the fixture and answers commands the way the
//! real engine will, so the UI can be built and tested without a server.

use std::path::Path;
use std::str::FromStr;

use model::{
    AccountId, Body, BodyRole, Command, Envelope, Event, FlagChange, FolderId, FolderRole, MailBackend,
    MailDate, MessageId, RequestId, StyledLine, StyledSpan, SyncStatus,
};

use crate::change_refusals::{DELETED_NOT_HERE, move_refusal};
use crate::mock::mutate::{MoveOutcome, apply_flags, delete_messages, move_messages};
use crate::mock::{raw_view, search};
use crate::mock::{account_state, select_account};
use crate::mock::{Fixture, FixtureError};

mod folder_commands;

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
        let mut events = vec![
            Event::HostInfo {
                host_name: self.data.host_name.clone(),
                system: self.data.system.clone(),
            },
            Event::AccountList(self.data.accounts.clone()),
        ];
        events.extend(account_state(&self.data));
        if let Some(folder) = self.data.open_folder() {
            events.push(self.envelope_page(folder));
            events.push(Event::BodyReady {
                message: self.data.open_message,
                body: self.data.open_body.clone(),
            });
        }
        events
    }

    fn folder_list(&self) -> Event {
        Event::FolderList {
            account: self.data.account.clone(),
            folders: self.data.folders.clone(),
        }
    }

    /// The folder list, and after it the account's Inbox unread count, which
    /// is what the account list shows, when the fixture gives the account a
    /// count (see `AccountRow::unread`).
    fn folder_list_events(&self) -> Vec<Event> {
        let mut events = vec![self.folder_list()];
        let counted = self.data.accounts.iter().any(|listed| listed.id == self.data.account && listed.unread.is_some());
        let inbox = self.data.folders.iter().find(|folder| folder.role == FolderRole::Inbox);
        events.extend(inbox.filter(|_| counted).map(|folder| Event::AccountUnread {
            account: self.data.account.clone(),
            unread: folder.unread,
        }));
        events
    }

    /// A reply was sent: its original is marked answered, as the real
    /// backend does once the server has accepted the message.
    fn mark_answered(&mut self, original: MessageId) -> Vec<Event> {
        let change = FlagChange { answered: Some(true), ..FlagChange::default() };
        let outcome = apply_flags(&mut self.data, &[original], change);
        if outcome.updates.is_empty() {
            return Vec::new();
        }
        vec![Event::FlagsChanged { updates: outcome.updates }]
    }

    /// The newest page of a folder: every fixture message filed there, newest
    /// first. The total is the folder's own count, which is larger than the
    /// page when the fixture holds only a sample; a folder with no messages
    /// is empty.
    fn envelope_page(&self, folder: FolderId) -> Event {
        if let Some(role) = self.data.folders.iter().find(|entry| entry.id == folder).map(|entry| entry.role).filter(|role| role.is_virtual()) {
            return self.saved_search_page(folder, role);
        }
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

    /// The real backend never changes the deleted flag this way: it says so
    /// and carries out the rest of the change.
    fn set_flags(&mut self, ids: &[MessageId], change: FlagChange) -> Vec<Event> {
        let deleted_asked = change.deleted.is_some();
        let change = FlagChange {
            deleted: None,
            ..change
        };
        let mut events = Vec::new();
        let mut unknown = Vec::new();
        if change != FlagChange::default() {
            let outcome = apply_flags(&mut self.data, ids, change);
            unknown = outcome.unknown;
            if !outcome.updates.is_empty() {
                events.push(Event::FlagsChanged {
                    updates: outcome.updates,
                });
            }
            if outcome.counts_moved {
                events.extend(self.folder_list_events());
            }
        }
        if deleted_asked {
            events.push(self.failure("set flags", DELETED_NOT_HERE.to_owned()));
        }
        events.extend(self.unknown_messages("set flags", &unknown));
        events
    }

    fn move_to(&mut self, ids: &[MessageId], to: FolderId) -> Vec<Event> {
        let Some(target) = self.data.folders.iter().find(|entry| entry.id == to) else {
            return vec![self.failure("move", format!("unknown folder id: {}", to.0))];
        };
        if let Some(reason) = move_refusal(target.role) {
            return vec![self.failure("move", reason.to_owned())];
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
            events.extend(self.folder_list_events());
            events.extend(
                outcome
                    .touched
                    .iter()
                    .map(|&folder| self.envelope_page(folder)),
            );
        }
        events.extend(self.unknown_messages(operation, &outcome.unknown));
        if let Some(reason) = outcome.refused {
            events.push(self.failure(operation, reason.to_owned()));
        }
        events
    }

    fn search(&self, request: RequestId, account: &AccountId, query: &str) -> Vec<Event> {
        if !self.data.accounts.iter().any(|entry| &entry.id == account) {
            return vec![self.failure("search", format!("unknown account: {}", account.0))];
        }
        let plan = match search::plan(query, &self.data.folders) {
            Ok(plan) => plan,
            Err(sentence) => return vec![self.failure("search", sentence)],
        };
        let envelopes: Vec<Envelope> = if *account == self.data.account {
            self.data
                .messages
                .iter()
                .filter(|envelope| plan.matches(envelope))
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        vec![Event::SearchResults {
            request,
            total: u32::try_from(envelopes.len()).unwrap_or(u32::MAX),
            envelopes,
            incomplete: None,
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
        reply_to: Vec::new(),
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
    /// Answers at once. A command the mock has nothing to say about
    /// (`Unlock`) gets no events; a send is accepted; a switch of account is
    /// answered as a backend with several accounts does (see
    /// `select_account`).
    fn handle(&mut self, command: Command) -> Vec<Event> {
        match command {
            Command::OpenFolder { folder } => self.open_folder(folder),
            // The mock's first page already holds every message it has.
            Command::LoadMore { .. } => Vec::new(),
            Command::FetchBody { message } => self.fetch_body(message),
            Command::FetchRaw { message, part } => raw_view::raw_event(&self.data, message, part)
                .map_or_else(|| vec![self.failure("view source", format!("unknown message id: {}", message.0))], |event| vec![event]),
            Command::PrepareHtml { .. } => vec![self.failure("open in browser", "this message has no HTML part to open".to_owned())],
            Command::FetchAttachment { .. } => vec![self.failure("save attachment", "the sample mailbox holds no attachments to save".to_owned())],
            Command::SetFlags { messages, change } => self.set_flags(&messages, change),
            Command::Move { messages, to } => self.move_to(&messages, to),
            Command::Delete { messages } => self.delete(&messages),
            change @ (Command::SetTag { .. }
            | Command::CreateFolder { .. }
            | Command::RenameFolder { .. }
            | Command::DeleteFolder { .. }
            | Command::MarkAllRead { .. }
            | Command::EmptyFolder { .. }) => self.change_command(change),
            Command::Search {
                request,
                account,
                query,
            } => self.search(request, &account, &query),
            Command::SyncNow { account } => self.sync_now(&account),
            Command::Send { message } => {
                let mut events = vec![Event::MessageSent { account: message.account }];
                events.extend(message.in_reply_to.map(|original| self.mark_answered(original)).unwrap_or_default());
                events
            }
            Command::SelectAccount { account } => select_account(&self.data, &account),
            Command::Unlock { .. } => Vec::new(),
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
