//! What a chosen menu entry does. The message entries do what their keys do,
//! on the same messages: the marked ones when there are any (the replies and
//! the copies take the selected message alone), and a delete asks first.

use model::{Command, RawPart};

use super::action::Action;
use super::menu_tables::MenuAction;
use super::menus::MenuTarget;
use crate::app_view::AppView;
use crate::input::compose::ComposeKind;
use crate::view_state::Focus;
use crate::OptionsAction;

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
            (MenuAction::ToggleAlert, _) => self.act(Action::ToggleAlert, grid),
            (MenuAction::Tag, _) => self.act(Action::Tag, grid),
            (MenuAction::NewFolder, MenuTarget::Folder(id)) => self.start_new_folder(Some(id)),
            (MenuAction::NewTopFolder, _) => self.start_new_folder(None),
            (MenuAction::RenameFolder, MenuTarget::Folder(id)) => self.start_rename(id),
            (MenuAction::DeleteFolder, MenuTarget::Folder(id)) => self.start_delete_folder(id),
            (MenuAction::MarkAllRead, MenuTarget::Folder(id)) => self.mark_folder_read(id),
            (MenuAction::EmptyFolder, MenuTarget::Folder(id)) => self.start_empty_folder(id),
            (MenuAction::Archive, _) => self.act(Action::Archive, grid),
            (MenuAction::Spam, _) => self.act(Action::Spam, grid),
            (MenuAction::CopySender, _) => self.copy_sender(),
            (MenuAction::CopySubject, _) => self.copy_subject(),
            (MenuAction::CopyText, _) => self.copy_message_text(grid),
            (MenuAction::FindInMessage, _) => self.find_in_message(),
            (MenuAction::ViewHeaders, _) => self.view_raw(RawPart::Headers),
            (MenuAction::ViewSource, _) => self.view_raw(RawPart::Source),
            (MenuAction::OpenInBrowser, _) => self.open_in_browser(),
            (MenuAction::OpenFolder, MenuTarget::Folder(id)) => {
                self.account_cursor = None;
                self.focus = Focus::Sidebar;
                self.change_folder(id);
            }
            (MenuAction::SyncNow, MenuTarget::Account(id)) => self.sync_account(id),
            (MenuAction::SyncNow, _) => self.refresh(),
            (MenuAction::EditAccount, MenuTarget::Account(id)) => self.queue_options_action(OptionsAction::EditAccount(id.0)),
            (MenuAction::OpenLink, MenuTarget::Link(address)) => self.open_link(address),
            (MenuAction::CopyLink, MenuTarget::Link(address)) => self.put_on_clipboard(Some("the link"), address),
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

    /// The first address the selected message is from.
    pub(crate) fn copy_sender(&mut self) {
        let sender = self.shown_page().and_then(|page| page.envelopes.get(self.selection.message_index)).and_then(|envelope| envelope.from.first());
        if let Some(address) = sender.map(|from| from.email.clone()) {
            self.put_on_clipboard(Some("the sender's address"), address);
        }
    }

    pub(crate) fn copy_subject(&mut self) {
        let subject = self.shown_page().and_then(|page| page.envelopes.get(self.selection.message_index)).map(|envelope| envelope.subject.clone());
        if let Some(subject) = subject {
            self.put_on_clipboard(Some("the subject"), subject);
        }
    }
}

#[cfg(test)]
include!("menu_run.tests.rs");
