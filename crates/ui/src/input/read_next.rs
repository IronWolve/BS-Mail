//! Going through a run of mail without going back to the list: `n` shows the
//! next message, `p` the previous one, and `N` the next unread one, from the
//! list or from the reader. Space in the reader reads on: a page down, and at
//! the end of the message the next unread one, as `N` finds it. The pane that
//! has focus keeps it; the selection, the list's scroll, the reader's scroll
//! (back to the top) and the timer that marks the message read all follow the
//! new message, because they all follow the selection.

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
        if self.focus == Focus::Sidebar || self.shown_page().is_none() {
            return;
        }
        match self.next_unread_row() {
            Some(row) => {
                let layout = self.layout(grid.0, grid.1);
                self.select_message(row, &layout);
            }
            None => self.mail.set_note(Some("No more unread messages in this list".to_string())),
        }
    }

    /// Space in the reader: a page down while the message has more to show,
    /// and at its end the next unread message, with the focus staying in the
    /// reader. With no unread message to go on to, the footer says so and
    /// the message stays. While there is nothing to read yet (the body has not
    /// arrived), it does nothing.
    pub(crate) fn read_on(&mut self, grid: (u16, u16)) {
        let layout = self.layout(grid.0, grid.1);
        let Some((lines, height)) = self.reader_extent(&layout) else {
            return;
        };
        if self.scroll.reader < lines.saturating_sub(height) {
            self.scroll_reader(Action::PageDown, &layout);
            return;
        }
        if self.shown_page().is_none() {
            return;
        }
        match self.next_unread_row() {
            Some(row) => self.select_message(row, &layout),
            None => self.mail.set_note(Some("No more unread here".to_string())),
        }
    }

    /// The row of the next unread message after the selected one, going on
    /// from the top of the list when there is none below; `None` when no
    /// other loaded message is unread. What `N` and Space at the end of a
    /// message both go to.
    fn next_unread_row(&self) -> Option<usize> {
        let page = self.shown_page()?;
        let (at, len) = (self.selection.message_index, page.envelopes.len());
        (at + 1..len).chain(0..at.min(len)).find(|&row| !page.envelopes[row].flags.seen)
    }
}

#[cfg(test)]
include!("read_next.tests.rs");
