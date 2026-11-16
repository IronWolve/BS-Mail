//! The mail data the regions draw, kept up to date from backend events.

use model::{
    Account, AccountId, Body, Envelope, Event, Folder, FolderId, MessageId, SyncStatus,
};

use super::page_merge::{append_older, merge_newest_without};
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
    /// The account the failure in `error` is about, when the backend said
    /// which. Only that account getting well, or its own answers, end it; the
    /// ones of the account on screen leave another account's failure alone.
    /// `None` for a note of the screen's own.
    pub error_account: Option<AccountId>,
    /// Messages the user asked to move or delete, whose rows leave the list
    /// with the newest page that follows. Older loaded rows the count says
    /// are gone are taken out by name from this.
    pub relocated: Vec<MessageId>,
    /// The account does not let the program change mail. A backend that
    /// never says so is taken to allow it.
    pub changes_refused: bool,
    /// Folders the user opened or closed, which a fresh folder list must not
    /// undo.
    pub expanded_by_user: std::collections::HashMap<FolderId, bool>,
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
            Event::AccountUnread { account, unread } => {
                if let Some(entry) = self.accounts.iter_mut().find(|listed| listed.id == account) {
                    entry.unread = Some(unread);
                }
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
                    self.apply_folders(folders, selection);
                }
            }
            Event::ConnectionState { account, state } => {
                if let Some(entry) = self.accounts.iter_mut().find(|a| a.id == account) {
                    entry.state = state;
                }
                // An account coming online ends its own failure, and the one
                // of the screen's own when it is the account in view; it says
                // nothing about another account's.
                let own = is_shown(selection, &account) || self.error_account.as_ref() == Some(&account);
                if state == model::ConnectionState::Online && own {
                    self.clear_failure(Some(&account));
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

    /// Whether a page of `folder` is one the screen takes. A page for a folder
    /// the user has already left is stale. So is a page for a folder this
    /// account does not have: after a switch of account the folder list is
    /// empty until the new account's arrives, and a page of the account that
    /// was left must not be shown meanwhile.
    pub fn accepts_page(&self, folder: FolderId, selection: &Selection) -> bool {
        let left = selection.folder.is_some_and(|chosen| chosen != folder);
        !left && self.folders.iter().any(|known| known.id == folder)
    }

    /// A page of a folder. The newest page read again keeps the older mail
    /// that was loaded below it; any other page replaces what is shown.
    fn apply_page(&mut self, folder: FolderId, offset: u32, total: u32, envelopes: Vec<Envelope>, selection: &mut Selection) {
        if !self.accepts_page(folder, selection) {
            return;
        }
        let shown = self.page.take().filter(|page| page.folder == folder && offset == 0);
        let envelopes = match shown {
            Some(shown) => self.merged_with(shown.envelopes, total, envelopes),
            None => {
                self.relocated.clear();
                envelopes
            }
        };
        selection.page_loaded(folder, &envelopes);
        self.clear_failure(selection.account.as_ref());
        self.page = Some(ListPage { folder, offset, total, envelopes });
    }

    /// The newest page put on top of the list that was shown. The messages
    /// named in `relocated` that are still below the page stay named, for a
    /// page that does not show the change yet; the others are settled.
    fn merged_with(&mut self, old: Vec<Envelope>, total: u32, new: Vec<Envelope>) -> Vec<Envelope> {
        let on_page: Vec<MessageId> = new.iter().map(|envelope| envelope.id).collect();
        let merged = merge_newest_without(old, total, new, &self.relocated);
        self.relocated
            .retain(|id| !on_page.contains(id) && merged.iter().any(|envelope| envelope.id == *id));
        merged
    }

    /// The folder tree of the account on screen. A folder that was open and is
    /// no longer there leaves nothing of its mail behind: the list, the
    /// message and its body go until the folder the selection moves to has
    /// its own page.
    fn apply_folders(&mut self, folders: Vec<Folder>, selection: &mut Selection) {
        self.folders = folders;
        for folder in &mut self.folders {
            if let Some(open) = self.expanded_by_user.get(&folder.id) {
                folder.expanded = *open;
            }
        }
        let before = selection.folder;
        selection.choose_folder(&self.folders);
        if selection.folder != before {
            self.page = None;
            self.body = None;
            self.relocated.clear();
            selection.message = None;
            selection.message_index = 0;
        }
    }

    /// Records what the user's own screen has to say (a note, a partial
    /// answer), replacing any failure shown. `None` clears it.
    pub fn set_note(&mut self, note: Option<String>) {
        self.error = note;
        self.error_account = None;
    }

    /// Ends the failure shown when it is about `account`, or is a note that
    /// is not about any. `None` is for an answer with no account in view.
    fn clear_failure(&mut self, account: Option<&AccountId>) {
        if self.error_account.is_none() || self.error_account.as_ref() == account {
            self.set_note(None);
        }
    }

    /// A failure the backend reported. One of an account that is not on screen
    /// says which account it is, since the footer holds only one line.
    fn note_failure(&mut self, account: Option<AccountId>, operation: &str, reason: &str, selection: &Selection) {
        let text = match account.as_ref().filter(|id| !is_shown(selection, id)) {
            Some(id) => {
                let name = self.account(id).map_or(id.0.as_str(), |known| known.name.as_str());
                format!("{name}: {operation}: {reason}")
            }
            None => format!("{operation}: {reason}"),
        };
        self.error = Some(text);
        self.error_account = account;
        if operation == "move" || operation == "delete" {
            self.relocated.clear();
        }
    }

    /// A page of older mail. It only means something next to the list it
    /// follows: the same folder, still open.
    fn apply_older(&mut self, folder: FolderId, total: u32, envelopes: Vec<Envelope>, selection: &Selection) {
        let Some(page) = self.page.as_mut().filter(|page| page.folder == folder) else {
            return;
        };
        if selection.folder != Some(folder) {
            return;
        }
        page.envelopes = append_older(std::mem::take(&mut page.envelopes), envelopes);
        page.total = total;
        self.clear_failure(selection.account.as_ref());
    }

    fn apply_list_event(&mut self, event: Event, selection: &mut Selection) {
        match event {
            Event::EnvelopePage { folder, offset, total, envelopes } => {
                self.apply_page(folder, offset, total, envelopes, selection);
            }
            Event::OlderEnvelopes { folder, total, envelopes, .. } => {
                self.apply_older(folder, total, envelopes, selection);
            }
            Event::BodyReady { message, body } => {
                self.clear_failure(selection.account.as_ref());
                self.body = Some((message, body));
            }
            Event::OpFailed { account, operation, reason } => {
                self.note_failure(account, &operation, &reason, selection);
            }
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
