//! Marking messages, so one key can act on many.
//!
//! Space marks the message under the cursor and moves on to the next, so a
//! run of Space marks a run of messages; `a` marks every message shown, `i`
//! inverts the marks and `A` or Escape clears them. The change keys act on the
//! marked messages when there are any, and on the selected one when there are
//! none. A marked row shows its number in reverse, and the list's title says
//! how many are marked.

use model::{Envelope, MessageId};
use surface::{Frame, Modifier};

use crate::app_view::AppView;
use crate::layout::{ColumnId, ListRegion};

impl AppView {
    /// Marks or unmarks the selected message.
    pub(crate) fn toggle_mark(&mut self) {
        let Some(id) = self.current_id() else {
            return;
        };
        if !self.marked.remove(&id) {
            self.marked.insert(id);
        }
    }

    /// Marks every message shown in the list (the ones loaded so far, which
    /// include the answer to a search while one is showing).
    pub(crate) fn mark_all(&mut self) {
        let ids: Vec<MessageId> = self.shown_page_for_marks().map(|page| page.iter().map(|e| e.id).collect()).unwrap_or_default();
        self.marked.extend(ids);
    }

    /// Marks the messages that are not marked and unmarks the ones that are.
    pub(crate) fn invert_marks(&mut self) {
        let ids: Vec<MessageId> = self.shown_page_for_marks().map(|page| page.iter().map(|e| e.id).collect()).unwrap_or_default();
        for id in ids {
            if !self.marked.remove(&id) {
                self.marked.insert(id);
            }
        }
    }

    /// The messages a change key acts on, in the order of the list: the
    /// marked ones, else the selected one.
    pub(crate) fn targets(&self) -> Vec<MessageId> {
        let Some(page) = self.shown_page_for_marks() else {
            return Vec::new();
        };
        if self.marked.is_empty() {
            return self.current_id().into_iter().collect();
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
        let present: Vec<MessageId> = self
            .shown_page_for_marks()
            .map(|page| page.iter().map(|e| e.id).collect())
            .unwrap_or_default();
        self.marked.retain(|id| present.contains(id));
    }

    fn current_id(&self) -> Option<MessageId> {
        self.shown_page_for_marks()?.get(self.selection.message_index).map(|e| e.id)
    }

    fn shown_page_for_marks(&self) -> Option<&[Envelope]> {
        let folder = self.selection.folder?;
        self.mail
            .page
            .as_ref()
            .filter(|page| page.folder == folder)
            .map(|page| page.envelopes.as_slice())
    }

    /// Shows the marked rows: their number cells in reverse, or, when the
    /// number column is not there (a narrow list, or one set up without it),
    /// the first cell that is.
    pub(crate) fn show_marks(&self, frame: &mut Frame, list: &ListRegion, first: usize, envelopes: &[Envelope]) {
        if self.marked.is_empty() {
            return;
        }
        let slots = &list.columns.slots;
        let Some(number) = slots.iter().find(|slot| slot.id == ColumnId::Num).or_else(|| slots.first()) else {
            return;
        };
        for (line, envelope) in envelopes.iter().skip(first).take(usize::from(list.rows.height)).enumerate() {
            if !self.marked.contains(&envelope.id) {
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
