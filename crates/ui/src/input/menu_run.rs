//! What a chosen menu entry does. The message entries do what their keys do,
//! on the same messages: the marked ones when there are any (the replies and
//! the copies take the selected message alone), and a delete asks first.

use model::Command;

use super::action::Action;
use super::menu_tables::MenuAction;
use super::menus::MenuTarget;
use crate::app_view::AppView;
use crate::input::compose::ComposeKind;
use crate::view_state::Focus;
use crate::OptionsAction;

/// What the footer says once something is copied.
const COPIED_NOTE: &str = "is on the clipboard (if your terminal allows it).";

impl AppView {
    /// Closes the menu and does what entry `index` stands for.
    pub(crate) fn run_menu_entry(&mut self, index: usize, grid: (u16, u16)) {
        let Some(open) = self.menu.take() else {
            return;
        };
        let Some(action) = open.menu.entries.get(index).map(|entry| entry.action) else {
            return;
        };
        self.run_menu_action(action, open.target, grid);
    }

    fn run_menu_action(&mut self, action: MenuAction, target: MenuTarget, grid: (u16, u16)) {
        match (action, target) {
            (MenuAction::Reply, _) => self.start_compose(ComposeKind::Reply),
            (MenuAction::ReplyAll, _) => self.start_compose(ComposeKind::ReplyAll),
            (MenuAction::Forward, _) => self.start_compose(ComposeKind::Forward),
            (MenuAction::EditAsNew, _) => self.start_compose(ComposeKind::EditAsNew),
            (MenuAction::Delete, _) => self.act(Action::Delete, grid),
            (MenuAction::Move, _) => self.act(Action::Move, grid),
            (MenuAction::ToggleRead, _) => self.act(Action::ToggleRead, grid),
            (MenuAction::ToggleStar, _) => self.act(Action::ToggleStar, grid),
            (MenuAction::Archive, _) => self.act(Action::Archive, grid),
            (MenuAction::Spam, _) => self.act(Action::Spam, grid),
            (MenuAction::CopySender, _) => self.copy_sender(),
            (MenuAction::CopySubject, _) => self.copy_subject(),
            (MenuAction::OpenFolder, MenuTarget::Folder(id)) => {
                self.account_cursor = None;
                self.focus = Focus::Sidebar;
                self.change_folder(id);
            }
            (MenuAction::SyncNow, MenuTarget::Account(id)) => self.sync_account(id),
            (MenuAction::SyncNow, _) => self.refresh(),
            (MenuAction::EditAccount, MenuTarget::Account(id)) => self.queue_options_action(OptionsAction::EditAccount(id.0)),
            (MenuAction::OpenLink, MenuTarget::Link(address)) => {
                self.mail.set_note(Some(format!("Opening {address}")));
                self.link_requests.push(address);
            }
            (MenuAction::CopyLink, MenuTarget::Link(address)) => self.copy_text("The link", address),
            _ => {}
        }
    }

    /// Asks for a sync of an account, which need not be the one on screen.
    fn sync_account(&mut self, id: model::AccountId) {
        if self.selection.account.as_ref() == Some(&id) {
            self.refresh();
        } else {
            self.queue.push(Command::SyncNow { account: id });
        }
    }

    /// Puts `text` on the clipboard, for the host to take, and says so.
    fn copy_text(&mut self, what: &str, text: String) {
        self.mail.set_note(Some(format!("{what} {COPIED_NOTE}")));
        self.copied_link = Some(text);
    }

    /// The first address the selected message is from.
    fn copy_sender(&mut self) {
        let sender = self.shown_page().and_then(|page| page.envelopes.get(self.selection.message_index)).and_then(|envelope| envelope.from.first());
        if let Some(address) = sender.map(|from| from.email.clone()) {
            self.copy_text("The sender's address", address);
        }
    }

    fn copy_subject(&mut self) {
        let subject = self.shown_page().and_then(|page| page.envelopes.get(self.selection.message_index)).map(|envelope| envelope.subject.clone());
        if let Some(subject) = subject {
            self.copy_text("The subject", subject);
        }
    }
}

#[cfg(test)]
include!("menu_run.tests.rs");
