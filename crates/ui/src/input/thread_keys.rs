//! Threads in the list: `T` turns them on or off for this session, Right
//! opens the thread whose row is selected and Left closes it (from one of its
//! messages, Left closes the thread and selects its row), and a click on the
//! arrow at the start of a thread's row does either. The setting that keeps
//! threads on between runs is `[list] threaded`, on the Appearance page.
//!
//! Only the loaded mail is grouped, and the list's title says so.

use model::MessageId;

use crate::app_view::AppView;
use crate::layout::{ColumnId, Layout};
use crate::list_order::RowKind;
use crate::view_state::ListPage;
use crate::widgets::list_rows::thread_cell::ARROW_CELLS;

impl AppView {
    /// `T`: threads on or off, for this session.
    pub(crate) fn toggle_threads(&mut self) {
        let on = !self.mail.arrangement.threaded;
        self.mail.arrangement.threaded = on;
        self.mail.arrangement.open.clear();
        self.arrangement_changed();
        let what = if on { "Threads on: the loaded mail is grouped" } else { "Threads off" };
        self.mail.done_text = Some(what.to_string());
    }

    /// Makes the rows again after the arrangement changed, and shows the
    /// selected message again if it has moved.
    pub(crate) fn arrangement_changed(&mut self) {
        self.mail.rearrange(&mut self.selection);
        self.scroll.list_detached = None;
    }

    /// Right on the list: opens the selected thread. False when the selected
    /// row is not a thread that is closed, so the key does what it always did.
    pub(crate) fn open_thread(&mut self) -> bool {
        let Some(members) = self.selected_thread_members(|kind| kind == RowKind::Head { open: false }) else {
            return false;
        };
        self.mail.arrangement.open.extend(members);
        self.arrangement_changed();
        true
    }

    /// Left on the list: closes the thread of the selected row, which is the
    /// row itself or the message it is open under. False when there is none.
    pub(crate) fn close_thread(&mut self) -> bool {
        let Some(members) = self.selected_thread_members(|kind| matches!(kind, RowKind::Head { open: true } | RowKind::Child)) else {
            return false;
        };
        self.close_members(&members);
        true
    }

    /// A click on the arrow of the thread's row `row`: opens it, or closes it.
    pub(crate) fn toggle_thread_at(&mut self, row: usize) {
        let Some(page) = self.shown_page() else {
            return;
        };
        let (kind, members) = thread_at(page, row);
        match (kind, members) {
            (RowKind::Head { open: false }, Some(members)) => {
                self.mail.arrangement.open.extend(members);
                self.arrangement_changed();
            }
            (RowKind::Head { open: true }, Some(members)) => self.close_members(&members),
            _ => {}
        }
    }

    /// Whether a press at screen column `col` of list row `row` is on the
    /// arrow of a thread's row.
    pub(crate) fn on_thread_arrow(&self, layout: &Layout, row: usize, col: u16) -> bool {
        let is_head = self.shown_page().is_some_and(|page| matches!(page.row_kind(row), RowKind::Head { .. }));
        let slots = layout.list.as_ref().map(|list| list.columns.slots.as_slice()).unwrap_or_default();
        let subject = slots.iter().find(|slot| slot.id == ColumnId::Subject);
        is_head && subject.is_some_and(|slot| col >= slot.x && col < slot.x.saturating_add(ARROW_CELLS))
    }

    /// Closes the thread with these messages; the selection follows its
    /// message to the thread's row.
    fn close_members(&mut self, members: &[MessageId]) {
        for id in members {
            self.mail.arrangement.open.remove(id);
        }
        self.arrangement_changed();
    }

    /// The messages of the thread of the selected row, when its kind is one
    /// `wanted` accepts.
    fn selected_thread_members(&self, wanted: impl Fn(RowKind) -> bool) -> Option<Vec<MessageId>> {
        if !self.mail.arrangement.threaded {
            return None;
        }
        let (kind, members) = thread_at(self.shown_page()?, self.selection.message_index);
        wanted(kind).then_some(members).flatten()
    }
}

/// The kind of row `row` and the messages of its thread, if it has one.
fn thread_at(page: &ListPage, row: usize) -> (RowKind, Option<Vec<MessageId>>) {
    (page.row_kind(row), page.thread_of_row(row).map(|thread| thread.members.clone()))
}

#[cfg(test)]
include!("thread_keys.tests.rs");
