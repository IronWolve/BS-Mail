//! Marking messages, so one key can act on many.
//!
//! Space marks the message under the cursor and moves on to the next, so a
//! run of Space marks a run of messages; `a` marks every message shown, `i`
//! inverts the marks and `A` or Escape clears them. The change keys act on the
//! marked messages when there are any, and on the selected one when there are
//! none. A marked row shows its number in reverse, and the list's title says
//! how many are marked.

use std::collections::HashSet;

use model::{Envelope, MessageId};
use surface::{Frame, Modifier};

use crate::app_view::AppView;
use crate::layout::{ColumnId, ListRegion};
use crate::list_order::RowKind;
use crate::view_state::ListPage;

impl AppView {
    /// Marks or unmarks the selected message: all of a thread's messages when
    /// the row is a thread's.
    pub(crate) fn toggle_mark(&mut self) {
        let ids = self.current_row_ids();
        self.toggle_marks_of(&ids);
    }

    /// Unmarks `ids` when any of them is marked, else marks them all. For one
    /// message that is a plain toggle.
    pub(crate) fn toggle_marks_of(&mut self, ids: &[MessageId]) {
        if ids.iter().any(|id| self.marked.contains(id)) {
            for id in ids {
                self.marked.remove(id);
            }
        } else {
            self.marked.extend(ids.iter().copied());
        }
    }

    /// Marks every message loaded for the list, the ones folded into a
    /// thread's row too (the answer to a search while one is showing).
    pub(crate) fn mark_all(&mut self) {
        let ids: Vec<MessageId> = self.loaded_for_marks().map(|page| page.iter().map(|e| e.id).collect()).unwrap_or_default();
        self.marked.extend(ids);
    }

    /// Marks the messages that are not marked and unmarks the ones that are.
    pub(crate) fn invert_marks(&mut self) {
        let ids: Vec<MessageId> = self.loaded_for_marks().map(|page| page.iter().map(|e| e.id).collect()).unwrap_or_default();
        for id in ids {
            if !self.marked.remove(&id) {
                self.marked.insert(id);
            }
        }
    }

    /// The messages a change key acts on, in the order they were loaded: the
    /// marked ones, else the selected one (every message of its thread when
    /// the selected row is a thread's).
    pub(crate) fn targets(&self) -> Vec<MessageId> {
        let Some(page) = self.loaded_for_marks() else {
            return Vec::new();
        };
        if self.marked.is_empty() {
            return self.current_row_ids();
        }
        page.iter().filter(|e| self.marked.contains(&e.id)).map(|e| e.id).collect()
    }

    pub(crate) fn clear_marks(&mut self) {
        self.marked.clear();
    }

    /// Forgets marks on messages that are no longer on the page, such as
    /// after a delete or a change of folder.
    pub(crate) fn prune_marks(&mut self) {
        if self.marked.is_empty() {
            return;
        }
        let present: HashSet<MessageId> =
            self.loaded_for_marks().map(|page| page.iter().map(|e| e.id).collect()).unwrap_or_default();
        self.marked.retain(|id| present.contains(id));
    }

    /// Every message loaded for the folder shown, whether or not a row of its
    /// own shows it.
    fn loaded_for_marks(&self) -> Option<&[Envelope]> {
        self.shown_page().map(|page| page.loaded.as_slice())
    }

    /// Shows the marked rows: their number cells in reverse, or, when the
    /// number column is not there (a narrow list, or one set up without it),
    /// the first cell that is. A thread's row is marked when any of its
    /// messages is.
    pub(crate) fn show_marks(&self, frame: &mut Frame, list: &ListRegion, first: usize, page: &ListPage) {
        if self.marked.is_empty() {
            return;
        }
        let slots = &list.columns.slots;
        let Some(number) = slots.iter().find(|slot| slot.id == ColumnId::Num).or_else(|| slots.first()) else {
            return;
        };
        let shown = page.envelopes.iter().enumerate().skip(first).take(usize::from(list.rows.height));
        for (line, (row, envelope)) in shown.enumerate() {
            let marked = match page.thread_of_row(row) {
                Some(thread) if matches!(page.row_kind(row), RowKind::Head { .. }) => {
                    thread.members.iter().any(|id| self.marked.contains(id))
                }
                _ => self.marked.contains(&envelope.id),
            };
            if !marked {
                continue;
            }
            let y = list.rows.y.saturating_add(u16::try_from(line).unwrap_or(u16::MAX));
            for x in number.x..number.x.saturating_add(number.width) {
                if let Some(cell) = frame.cells.cell_mut((x, y)) {
                    cell.modifier.insert(Modifier::REVERSED);
                }
            }
        }
    }
}

#[cfg(test)]
include!("marks.tests.rs");
