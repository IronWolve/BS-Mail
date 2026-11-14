//! The mail data the regions draw, kept up to date from backend events.

use model::{
    Account, AccountId, Body, Envelope, Event, Folder, FolderId, MessageId, SyncStatus,
};

use super::selection::Selection;

/// The machine the engine runs on, for the strip's host segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostSummary {
    pub name: String,
    pub system: String,
}

/// One page of the open folder's envelopes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListPage {
    pub folder: FolderId,
    pub offset: u32,
    /// Messages in the folder.
    pub total: u32,
    pub envelopes: Vec<Envelope>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MailSnapshot {
    pub accounts: Vec<Account>,
    pub host: Option<HostSummary>,
    /// The folders of the selected account, depth first.
    pub folders: Vec<Folder>,
    /// Sync state and storage of the selected account.
    pub status: SyncStatus,
    pub page: Option<ListPage>,
    pub body: Option<(MessageId, Body)>,
    /// The last failure the backend reported, until something succeeds.
    pub error: Option<String>,
    /// The account does not let the program change mail. A backend that
    /// never says so is taken to allow it.
    pub changes_refused: bool,
}

impl MailSnapshot {
    pub fn folder(&self, id: FolderId) -> Option<&Folder> {
        self.folders.iter().find(|folder| folder.id == id)
    }

    pub fn account(&self, id: &AccountId) -> Option<&Account> {
        self.accounts.iter().find(|account| &account.id == id)
    }

    /// Stores what `event` carries for the regions and moves the selection
    /// where the event calls for it (first account, Inbox, first message).
    /// Events for an account that is not selected are not shown, except its
    /// connection state.
    pub fn apply(&mut self, event: Event, selection: &mut Selection) {
        match event {
            Event::AccountList(accounts) => {
                self.accounts = accounts;
                selection.choose_account(&self.accounts);
            }
            Event::ChangesAllowed { account, allowed } => {
                if is_shown(selection, &account) {
                    self.changes_refused = !allowed;
                }
            }
            Event::HostInfo { host_name, system } => {
                self.host = Some(HostSummary {
                    name: host_name,
                    system,
                });
            }
            Event::FolderList { account, folders } => {
                if is_shown(selection, &account) {
                    self.folders = folders;
                    selection.choose_folder(&self.folders);
                }
            }
            Event::ConnectionState { account, state } => {
                if let Some(entry) = self.accounts.iter_mut().find(|a| a.id == account) {
                    entry.state = state;
                }
                if state == model::ConnectionState::Online {
                    self.error = None;
                }
            }
            Event::Quota { account, quota } => {
                if is_shown(selection, &account) {
                    self.status.quota = quota;
                }
            }
            Event::SyncProgress { account, status } => {
                if is_shown(selection, &account) {
                    let quota = status.quota.or(self.status.quota);
                    self.status = status;
                    self.status.quota = quota;
                }
            }
            other => self.apply_list_event(other, selection),
        }
    }

    fn apply_list_event(&mut self, event: Event, selection: &mut Selection) {
        match event {
            Event::EnvelopePage {
                folder,
                offset,
                total,
                envelopes,
            } => {
                // A page for a folder the user has already left is stale.
                if selection.folder.is_some_and(|chosen| chosen != folder) {
                    return;
                }
                selection.page_loaded(folder, &envelopes);
                self.error = None;
                self.page = Some(ListPage {
                    folder,
                    offset,
                    total,
                    envelopes,
                });
            }
            Event::BodyReady { message, body } => {
                self.error = None;
                self.body = Some((message, body));
            }
            Event::OpFailed {
                operation, reason, ..
            } => self.error = Some(format!("{operation}: {reason}")),
            Event::FlagsChanged { updates } => {
                let Some(page) = self.page.as_mut() else {
                    return;
                };
                for (id, flags) in updates {
                    if let Some(envelope) = page.envelopes.iter_mut().find(|e| e.id == id) {
                        envelope.flags = flags;
                    }
                }
            }
            _ => {}
        }
    }
}

/// An account's data is shown when it is the selected one, or none is yet.
fn is_shown(selection: &Selection, account: &AccountId) -> bool {
    selection.account.as_ref().is_none_or(|shown| shown == account)
}

#[cfg(test)]
include!("mail_snapshot.tests.rs");
