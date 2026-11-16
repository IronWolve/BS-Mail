//! The page of the open folder: the messages loaded so far, and the rows the
//! list draws from them.
//!
//! `loaded` holds the messages in the order the engine gave them, which is
//! what a newer or an older page merges into. `envelopes` holds the rows the
//! list draws, made from them by the arrangement (sorted, and with threads
//! folded into one row each). Every row of the list, and the selection's
//! `message_index`, is a place in `envelopes`; the keys that change mail
//! find their messages in `loaded`, since a folded thread's other messages
//! have no row.

use model::{Envelope, Folder, FolderId, MessageId};

use crate::list_order::arrange::arrange;
use crate::list_order::sort::sorts_by_recipient;
use crate::list_order::{Arrangement, RoleKey, RowKind, RowMeta, ThreadInfo};
use crate::widgets::list_rows::party::role_of;

/// One page of the open folder's envelopes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListPage {
    pub folder: FolderId,
    pub offset: u32,
    /// Messages in the folder.
    pub total: u32,
    /// The rows the list draws, in order.
    pub envelopes: Vec<Envelope>,
    /// Every loaded message, in the engine's order.
    pub loaded: Vec<Envelope>,
    /// What each row is, one for each of `envelopes`; empty when every row is
    /// a message on its own.
    pub rows: Vec<RowMeta>,
    pub threads: Vec<ThreadInfo>,
}

impl ListPage {
    /// A page whose rows are the loaded messages as they are.
    pub fn new(folder: FolderId, offset: u32, total: u32, loaded: Vec<Envelope>) -> Self {
        Self { folder, offset, total, envelopes: loaded.clone(), loaded, rows: Vec::new(), threads: Vec::new() }
    }

    /// A page arranged as `arrangement` says, for the folders `folders` (the
    /// folder's kind picks the sort).
    pub fn arranged(
        folder: FolderId,
        offset: u32,
        total: u32,
        loaded: Vec<Envelope>,
        arrangement: &Arrangement,
        folders: &[Folder],
    ) -> Self {
        let mut page = Self::new(folder, offset, total, loaded);
        page.arrange(arrangement, folders);
        page
    }

    /// Makes the rows from the loaded messages again.
    pub fn arrange(&mut self, arrangement: &Arrangement, folders: &[Folder]) {
        let role = folders.iter().find(|candidate| candidate.id == self.folder).map(|folder| folder.role).unwrap_or_default();
        let sort = arrangement.sort_for(RoleKey::of(role));
        let recipients = |envelope: &Envelope| sorts_by_recipient(role_of(folders, envelope));
        let made = arrange(&self.loaded, sort, arrangement, &recipients);
        self.envelopes = made.envelopes;
        self.rows = made.rows;
        self.threads = made.threads;
    }

    /// What row `row` is.
    pub fn row_meta(&self, row: usize) -> RowMeta {
        self.rows.get(row).copied().unwrap_or(RowMeta::SINGLE)
    }

    pub fn row_kind(&self, row: usize) -> RowKind {
        self.row_meta(row).kind
    }

    /// The thread a row belongs to, when it is a thread's row or one of its
    /// messages.
    pub fn thread_of_row(&self, row: usize) -> Option<&ThreadInfo> {
        let meta = self.rows.get(row)?;
        (meta.kind != RowKind::Single).then(|| self.threads.get(meta.thread)).flatten()
    }

    /// The messages row `row` stands for: every message of its thread when it
    /// is a thread's row, otherwise its own.
    pub fn row_ids(&self, row: usize) -> Vec<MessageId> {
        match (self.row_kind(row), self.thread_of_row(row)) {
            (RowKind::Head { .. }, Some(thread)) => thread.members.clone(),
            _ => self.envelopes.get(row).map(|envelope| envelope.id).into_iter().collect(),
        }
    }

    /// The row that shows message `id`: its own, or the row of the thread it
    /// is folded into.
    pub fn row_of(&self, id: MessageId) -> Option<usize> {
        if let Some(row) = self.envelopes.iter().position(|envelope| envelope.id == id) {
            return Some(row);
        }
        let thread = self.threads.iter().position(|thread| thread.members.contains(&id))?;
        self.rows.iter().position(|meta| meta.thread == thread && matches!(meta.kind, RowKind::Head { .. }))
    }

    /// The loaded message `id`, whether or not it has a row of its own.
    pub fn loaded_envelope(&self, id: MessageId) -> Option<&Envelope> {
        self.loaded.iter().find(|envelope| envelope.id == id)
    }

    /// Whether the folder's older mail is all loaded.
    pub fn all_loaded(&self) -> bool {
        u64::from(self.total) <= u64::from(self.offset) + self.loaded.len() as u64
    }
}

#[cfg(test)]
include!("list_page.tests.rs");
