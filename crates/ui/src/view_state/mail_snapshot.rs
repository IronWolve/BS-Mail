//! The mail data the regions draw, kept up to date from backend events.

use std::collections::{BTreeSet, HashMap, HashSet};

use model::{
    Account, AccountId, Body, Event, Folder, FolderId, MessageId, SyncStatus,
};

use super::list_page::ListPage;
use crate::list_order::Arrangement;
use super::selection::Selection;

/// The machine the engine runs on, for the strip's host segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostSummary {
    pub name: String,
    pub system: String,
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
    /// The accounts that said they do not let the program change mail, shown
    /// or not. An account that never said so is taken to allow it. The
    /// composer writes only from the others.
    pub read_only: HashSet<AccountId>,
    /// The signature each account's file asks for; an account with none has
    /// no entry.
    pub signatures: HashMap<AccountId, String>,
    /// Folders the user opened or closed, which a fresh folder list must not
    /// undo.
    pub expanded_by_user: std::collections::HashMap<FolderId, bool>,
    /// The tags seen on each account's mail (see `known_tags`).
    pub known_tags: HashMap<AccountId, BTreeSet<String>>,
    /// A problem with an account's setup, which no later success ends: the
    /// footer keeps it under any other failure (see `note_failure`).
    pub setup_problem: Option<String>,
    /// What the footer says of a folder action or a tag that is done ("Made
    /// the folder Notes"): the page that follows does not end it, a failure or
    /// the next key does (see `done_note`).
    pub done_text: Option<String>,
    /// How the list is arranged: the sorts, threads and the open threads. The
    /// rows of the page are made from the loaded messages by it.
    pub arrangement: Arrangement,
}

impl MailSnapshot {
    pub fn folder(&self, id: FolderId) -> Option<&Folder> {
        self.folders.iter().find(|folder| folder.id == id)
    }

    pub fn account(&self, id: &AccountId) -> Option<&Account> {
        self.accounts.iter().find(|account| &account.id == id)
    }

    /// Whether mail may be written from the account: it did not say it is
    /// read-only.
    pub fn may_send_from(&self, id: &AccountId) -> bool {
        !self.read_only.contains(id)
    }

    /// The account's signature, or an empty text for none.
    pub fn signature_of(&self, id: &AccountId) -> &str {
        self.signatures.get(id).map_or("", String::as_str)
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
            note @ (Event::ChangesAllowed { .. } | Event::AccountSignature { .. }) => {
                self.apply_account_facts(note, selection);
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

    /// What an account says about itself: whether it may send (also kept for
    /// the account on screen, which the keys that change mail look at) and its
    /// signature. Both are kept for every account, shown or not.
    fn apply_account_facts(&mut self, event: Event, selection: &Selection) {
        match event {
            Event::ChangesAllowed { account, allowed } => {
                if is_shown(selection, &account) {
                    self.changes_refused = !allowed;
                }
                if allowed {
                    self.read_only.remove(&account);
                } else {
                    self.read_only.insert(account);
                }
            }
            Event::AccountSignature { account, text } => {
                if text.is_empty() {
                    self.signatures.remove(&account);
                } else {
                    self.signatures.insert(account, text);
                }
            }
            _ => {}
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
    pub(super) fn clear_failure(&mut self, account: Option<&AccountId>) {
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
        self.done_text = None;
        if operation == "folders" {
            // The account file names a folder the server does not have: said
            // once, and kept, because the next page or body would end a
            // failure before it could be read.
            self.setup_problem = Some(text);
            return;
        }
        self.error = Some(text);
        self.error_account = account;
        if operation == "move" || operation == "delete" {
            self.relocated.clear();
        }
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
            Event::FlagsChanged { updates } => self.apply_flags(updates, selection),
            Event::TagsChanged { updates } => self.apply_tags(updates, selection),
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
