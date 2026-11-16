//! The accounts from the keyboard: in the sidebar, Down (`j`) on the last
//! folder runs on into the account rows, Up (`k`) on the first account comes
//! back to the folders, and Enter (or Right) on an account switches to it.
//! The row the cursor is on is drawn like a selected row and the Accounts
//! title is underlined as the focused one; Left and Escape leave the rows.
//!
//! The cursor only exists while the sidebar has focus and the account list
//! shows at least two accounts (with one there is nothing to switch to).

use super::action::Action;
use crate::app_view::AppView;
use crate::layout::{Layout, SectionKind};
use crate::view_state::Focus;
use crate::widgets::visible_rows;

impl AppView {
    /// How many account rows the sidebar shows, as many as fit.
    fn account_rows_showing(&self, layout: &Layout) -> usize {
        let room = layout
            .sidebar
            .as_ref()
            .and_then(|sidebar| sidebar.section(SectionKind::Accounts))
            .map_or(0, |section| usize::from(section.body.height));
        room.min(self.mail.accounts.len())
    }

    /// The account row the keyboard cursor is on, while there is one to show.
    pub(crate) fn account_cursor_row(&self) -> Option<usize> {
        let row = self.account_cursor.filter(|_| self.focus == Focus::Sidebar)?;
        (row < self.mail.accounts.len()).then_some(row)
    }

    /// A movement in the folders: when it is Down from the last folder, the
    /// cursor goes on into the accounts instead. True when it did.
    pub(crate) fn run_on_into_accounts(&mut self, movement: Action, layout: &Layout) -> bool {
        if movement != Action::Down || self.mail.accounts.len() < 2 || self.account_rows_showing(layout) == 0 {
            return false;
        }
        let rows = visible_rows(&self.mail.folders);
        let on_last = rows.last().is_none_or(|row| Some(row.folder.id) == self.selection.folder);
        if on_last {
            self.account_cursor = Some(0);
        }
        on_last
    }

    /// A key while the cursor is on an account row. True when it was used
    /// here; a movement toward the folders is left to them after the cursor
    /// has come back.
    pub(crate) fn account_cursor_key(&mut self, action: Action, grid: (u16, u16)) -> bool {
        let Some(row) = self.account_cursor_row() else {
            self.account_cursor = None;
            return false;
        };
        let layout = self.layout(grid.0, grid.1);
        let showing = self.account_rows_showing(&layout).max(1);
        match action {
            Action::Down => self.account_cursor = Some((row + 1).min(showing - 1)),
            Action::PageDown | Action::HalfPageDown | Action::Last => self.account_cursor = Some(showing - 1),
            Action::Up if row > 0 => self.account_cursor = Some(row - 1),
            Action::Up => self.account_cursor = None,
            Action::PageUp | Action::HalfPageUp | Action::First => {
                self.account_cursor = None;
                return false;
            }
            Action::Enter | Action::Right => self.switch_to_cursor(row),
            Action::Left | Action::Back => self.account_cursor = None,
            _ => return false,
        }
        true
    }

    /// Enter on an account row: switches to it. The cursor goes back to the
    /// folders, which are the new account's once they arrive. On the account
    /// already showing, nothing changes.
    fn switch_to_cursor(&mut self, row: usize) {
        if self.click_account(row) {
            self.account_cursor = None;
        }
    }
}

#[cfg(test)]
include!("account_cursor.tests.rs");
