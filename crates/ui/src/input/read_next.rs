//! Going through a run of mail without going back to the list: `n` shows the
//! next message, `p` the previous one, and `N` the next unread one, from the
//! list or from the reader. The pane that has focus keeps it; the selection,
//! the list's scroll, the reader's scroll (back to the top) and the timer that
//! marks the message read all follow the new message, because they all follow
//! the selection.

use super::action::Action;
use crate::app_view::AppView;
use crate::view_state::Focus;

impl AppView {
    /// `n` and `p`: the next or previous message of the list. At either end
    /// the selection stays, and the footer says so once the whole folder is
    /// loaded (while older mail is still to come, the end is not the end).
    pub(crate) fn show_neighbor(&mut self, movement: Action, grid: (u16, u16)) {
        if self.focus == Focus::Sidebar {
            return;
        }
        let Some(page) = self.shown_page() else {
            return;
        };
        let (at, len) = (self.selection.message_index, page.envelopes.len());
        let all_loaded = u64::from(page.offset) + len as u64 >= u64::from(page.total);
        let (at_end, words) = match movement {
            Action::Up => (at == 0, "the first"),
            _ => (at + 1 >= len, "the last"),
        };
        if at_end {
            if all_loaded {
                self.mail.set_note(Some(format!("This is {words} message in the list")));
            }
            return;
        }
        let layout = self.layout(grid.0, grid.1);
        self.move_message(movement, &layout);
    }

    /// `N`: the next unread message after the selected one, going on from the
    /// top of the list when there is none below. Only the messages loaded so
    /// far are looked at.
    pub(crate) fn show_next_unread(&mut self, grid: (u16, u16)) {
        if self.focus == Focus::Sidebar {
            return;
        }
        let Some(page) = self.shown_page() else {
            return;
        };
        let (at, len) = (self.selection.message_index, page.envelopes.len());
        let next = (at + 1..len).chain(0..at.min(len)).find(|&row| !page.envelopes[row].flags.seen);
        match next {
            Some(row) => {
                let layout = self.layout(grid.0, grid.1);
                self.select_message(row, &layout);
            }
            None => self.mail.set_note(Some("No more unread messages in this list".to_string())),
        }
    }
}

#[cfg(test)]
include!("read_next.tests.rs");
