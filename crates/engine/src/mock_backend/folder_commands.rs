//! The mock's answers to the folder actions and to a tag change: the same
//! commands the real backend takes, with the same events (the folder list
//! again, the folder's page again, the new tags) and the same refusals.

use model::{Command, Event, FolderId, FolderRole};

use super::MockBackend;
use crate::mock::folder_actions;

/// The operation names the real backend uses in its failure notices.
fn operation_of(command: &Command) -> &'static str {
    match command {
        Command::SetTag { .. } => "tag",
        Command::CreateFolder { .. } => "new folder",
        Command::RenameFolder { .. } => "rename folder",
        Command::DeleteFolder { .. } => "delete folder",
        Command::MarkAllRead { .. } => "mark all read",
        _ => "empty folder",
    }
}

impl MockBackend {
    /// Carries out a folder action or a tag change. Any other command gets no
    /// events.
    pub(super) fn change_command(&mut self, command: Command) -> Vec<Event> {
        let operation = operation_of(&command);
        match command {
            Command::SetTag { messages, tag, on } => self.tag(&messages, &tag, on),
            Command::CreateFolder { account, parent, name } => {
                let done = folder_actions::create_folder(&mut self.data, &account, parent, &name);
                self.folder_tree_changed(operation, done)
            }
            Command::RenameFolder { folder, name } => {
                let done = folder_actions::rename_folder(&mut self.data, folder, &name);
                self.folder_tree_changed(operation, done)
            }
            Command::DeleteFolder { folder } => {
                let done = folder_actions::delete_folder(&mut self.data, folder);
                self.folder_tree_changed(operation, done)
            }
            Command::MarkAllRead { folder } => match folder_actions::mark_all_read(&mut self.data, folder) {
                Ok(updates) => self.counts_and_page(folder, updates),
                Err(reason) => vec![self.failure(operation, reason)],
            },
            Command::EmptyFolder { folder } => match folder_actions::empty_folder(&mut self.data, folder) {
                Ok(()) => self.counts_and_page(folder, Vec::new()),
                Err(reason) => vec![self.failure(operation, reason)],
            },
            _ => Vec::new(),
        }
    }

    /// The folder list again, or the refusal.
    fn folder_tree_changed(&self, operation: &str, done: Result<(), String>) -> Vec<Event> {
        match done {
            Ok(()) => self.folder_list_events(),
            Err(reason) => vec![self.failure(operation, reason)],
        }
    }

    /// After a change to every message of a folder: the flags that changed,
    /// the folder list with its new counts, and the folder's page again.
    fn counts_and_page(&self, folder: FolderId, updates: Vec<(model::MessageId, model::Flags)>) -> Vec<Event> {
        let mut events = Vec::new();
        if !updates.is_empty() {
            events.push(Event::FlagsChanged { updates });
        }
        events.extend(self.folder_list_events());
        events.push(self.envelope_page(folder));
        events
    }

    fn tag(&mut self, ids: &[model::MessageId], tag: &str, on: bool) -> Vec<Event> {
        match folder_actions::set_tag(&mut self.data, ids, tag, on) {
            Ok(outcome) => {
                let mut events = Vec::new();
                if !outcome.updates.is_empty() {
                    events.push(Event::TagsChanged { updates: outcome.updates });
                }
                events.extend(self.unknown_messages("tag", &outcome.unknown));
                events
            }
            Err(reason) => vec![self.failure("tag", reason)],
        }
    }

    /// The page of a saved search: the messages of the whole fixture that are
    /// starred, or carry the alert marker, newest first.
    pub(super) fn saved_search_page(&self, folder: FolderId, role: FolderRole) -> Event {
        let mut envelopes: Vec<model::Envelope> = self
            .data
            .messages
            .iter()
            .filter(|envelope| if role == FolderRole::Starred { envelope.flags.flagged } else { envelope.flags.alert })
            .cloned()
            .collect();
        envelopes.sort_by(|a, b| b.date.unix.cmp(&a.date.unix));
        let total = u32::try_from(envelopes.len()).unwrap_or(u32::MAX);
        Event::EnvelopePage { folder, offset: 0, total, envelopes }
    }
}
