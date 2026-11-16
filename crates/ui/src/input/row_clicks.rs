//! What a press on a message row does, by where on the row and with which
//! key held.
//!
//! A press in the flag cell toggles read, one in the star cell toggles the
//! star, of that message alone and without moving the selection. Ctrl toggles
//! the row's mark; Shift marks the rows from the last one clicked to this
//! one. A plain press selects the row, and a second press on it within
//! `DOUBLE_CLICK_MS` opens it in the reader. A plain press also takes the row
//! (or the marked rows, when it is one of them) up for a drag onto a folder.

use model::MessageId;
use surface::Modifiers;

use super::compose::ComposeKind;
use super::memory::{DOUBLE_CLICK_MS, RowDrag};
use crate::app_view::AppView;
use crate::layout::{ColumnId, Layout};
use crate::view_state::Focus;

impl AppView {
    /// A left press on message row `index`, at cell `at`.
    pub(crate) fn press_message(&mut self, index: usize, at: (u16, u16), modifiers: Modifiers, layout: &Layout) {
        let Some(id) = self.shown_page().and_then(|page| page.envelopes.get(index)).map(|envelope| envelope.id) else {
            return;
        };
        if modifiers.ctrl {
            self.focus = Focus::List;
            self.toggle_marks_of(&self.row_ids_of(index));
            self.memory.anchor = Some(id);
        } else if modifiers.shift {
            self.focus = Focus::List;
            self.mark_range_to(index);
            self.memory.anchor = Some(id);
            self.select_message(index, layout);
        } else {
            match self.message_cell(layout, at.0) {
                Some(ColumnId::Flag) => self.toggle_read_of(&self.row_ids_of(index)),
                Some(ColumnId::Star) => self.toggle_star_of(&self.row_ids_of(index)),
                _ if self.on_thread_arrow(layout, index, at.0) => self.toggle_thread_at(index),
                _ => self.press_plain(index, id, at, layout),
            }
        }
    }

    /// The column of the message list at screen column `col`, if any.
    pub(crate) fn message_cell(&self, layout: &Layout, col: u16) -> Option<ColumnId> {
        let list = layout.list.as_ref()?;
        list.columns.slots.iter().find(|slot| col >= slot.x && col < slot.x.saturating_add(slot.width)).map(|slot| slot.id)
    }

    fn press_plain(&mut self, index: usize, id: MessageId, at: (u16, u16), layout: &Layout) {
        // A second click on the draft that is selected goes on with it.
        let again = self.focus == Focus::List && index == self.selection.message_index && self.selected_is_draft();
        let double = self.is_double_press(id);
        self.focus = Focus::List;
        self.select_message(index, layout);
        self.memory.anchor = Some(id);
        if again {
            self.start_compose(ComposeKind::Draft);
        } else if double {
            self.focus = Focus::Reader;
        } else {
            self.hold_rows(at, index);
        }
    }

    /// Whether this press on `id` is the second of a double-click, and what
    /// the next one is to be compared with.
    fn is_double_press(&mut self, id: MessageId) -> bool {
        let now = self.clock_ms;
        let double = self.memory.last_press.is_some_and(|(last, at)| last == id && now.saturating_sub(at) <= DOUBLE_CLICK_MS);
        self.memory.last_press = if double { None } else { Some((id, now)) };
        double
    }

    /// Takes the rows a drag from here would move: the marked ones when this
    /// row is one of them, else the messages this row stands for.
    fn hold_rows(&mut self, at: (u16, u16), index: usize) {
        let own = self.row_ids_of(index);
        let ids = if own.iter().any(|id| self.marked.contains(id)) { self.targets() } else { own };
        self.memory.row_drag = Some(RowDrag { pressed: at, ids, active: false, over: None });
    }

    /// Marks every row from the last one clicked (the selected row when none
    /// was) to row `index`, and the ends too.
    fn mark_range_to(&mut self, index: usize) {
        let Some(page) = self.shown_page() else {
            return;
        };
        let from = self
            .memory
            .anchor
            .and_then(|id| page.row_of(id))
            .unwrap_or(self.selection.message_index);
        let (low, high) = (from.min(index), from.max(index));
        let ids: Vec<MessageId> = (low..=high).flat_map(|row| page.row_ids(row)).collect();
        self.marked.extend(ids);
    }

    /// Tells the view what the caller's clock says now, in milliseconds, so
    /// two presses can be told from a double-click between ticks.
    pub fn set_clock(&mut self, now_ms: u64) {
        self.clock_ms = self.clock_ms.max(now_ms);
    }
}

#[cfg(test)]
include!("row_clicks.tests.rs");
