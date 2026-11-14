//! Switching between accounts, and forgetting mail state.
//!
//! A click on an account row in the sidebar, or the Switch button on the
//! Options page, makes another account the one whose mail shows. What the
//! screen knew of the old account is dropped at once (its folders, list,
//! message, search, marks and scroll), the list of accounts is kept, and the
//! backend is asked to send what the screen needs for the new one.

use model::{AccountId, Command, SyncStatus};

use crate::app_view::AppView;
use crate::input::CommandQueue;
use crate::view_state::{Focus, MailSnapshot, ScrollState, Selection};

impl AppView {
    /// A click on row `index` of the sidebar's accounts. Returns whether
    /// anything changed: clicking the account already showing does nothing.
    pub(crate) fn click_account(&mut self, index: usize) -> bool {
        let Some(id) = self.mail.accounts.get(index).map(|account| account.id.clone()) else {
            return false;
        };
        if !self.switch_account(id) {
            return false;
        }
        self.focus = Focus::Sidebar;
        true
    }

    /// Makes `id` the account whose mail shows. Returns false, changing
    /// nothing, when it already is.
    pub(crate) fn switch_account(&mut self, id: AccountId) -> bool {
        if self.selection.account.as_ref() == Some(&id) {
            return false;
        }
        self.forget_account_mail();
        self.selection.account = Some(id.clone());
        self.queue.push(Command::SelectAccount { account: id });
        true
    }

    /// Drops everything the screen holds about the mail of the account that
    /// was showing, but keeps the list of accounts.
    fn forget_account_mail(&mut self) {
        self.mail.folders.clear();
        self.mail.page = None;
        self.mail.body = None;
        self.mail.error = None;
        self.mail.status = SyncStatus::default();
        self.mail.changes_refused = false;
        self.mail.expanded_by_user.clear();
        self.selection.folder = None;
        self.selection.message = None;
        self.selection.message_index = 0;
        self.scroll = ScrollState::default();
        self.search = None;
        self.search_bar = None;
        self.marked.clear();
        self.picker = None;
        self.read_timer = None;
        self.auto_marked = None;
        self.queue.forget();
    }

    /// Forgets all mail state: accounts, folders, list, message, selection,
    /// scroll, search, marks and anything open over them. For when the
    /// account changed or was removed; the backend's next account list
    /// selects again. The name of the machine stays.
    pub fn clear_mail(&mut self) {
        let host = self.mail.host.take();
        self.mail = MailSnapshot::default();
        self.mail.host = host;
        self.selection = Selection::default();
        self.scroll = ScrollState::default();
        self.search = None;
        self.search_bar = None;
        self.marked.clear();
        self.picker = None;
        self.composer = None;
        self.read_timer = None;
        self.auto_marked = None;
        self.drag = None;
        self.queue = CommandQueue::default();
    }
}

#[cfg(test)]
include!("accounts.tests.rs");
