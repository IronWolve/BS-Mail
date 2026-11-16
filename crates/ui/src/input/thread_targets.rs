//! The messages a row of the list stands for. A message on its own, and a
//! message of an open thread, stand for themselves; the row of a thread
//! stands for every loaded message of the thread, so marking it, deleting it,
//! moving it or tagging it acts on all of them.

use model::MessageId;

use crate::app_view::AppView;

impl AppView {
    /// The messages the selected row stands for.
    pub(crate) fn current_row_ids(&self) -> Vec<MessageId> {
        self.shown_page().map_or_else(Vec::new, |page| page.row_ids(self.selection.message_index))
    }

    /// The messages row `row` stands for.
    pub(crate) fn row_ids_of(&self, row: usize) -> Vec<MessageId> {
        self.shown_page().map_or_else(Vec::new, |page| page.row_ids(row))
    }
}

#[cfg(test)]
include!("thread_targets.tests.rs");
