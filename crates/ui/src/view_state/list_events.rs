//! What the page of the open folder does with the mail events: a newest page,
//! a page of older mail, new flags. Each changes the loaded messages and then
//! makes the rows the list draws from them again, and the selection stays on
//! the message it was on, whatever row that message has now.

use model::{Envelope, FolderId, Flags, MessageId};

use super::list_page::ListPage;
use super::mail_snapshot::MailSnapshot;
use super::page_merge::{append_older, merge_newest_without};
use super::selection::Selection;

impl MailSnapshot {
    /// A page of a folder. The newest page read again keeps the older mail
    /// that was loaded below it; any other page replaces what is shown.
    pub(super) fn apply_page(&mut self, folder: FolderId, offset: u32, total: u32, envelopes: Vec<Envelope>, selection: &mut Selection) {
        if !self.accepts_page(folder, selection) {
            return;
        }
        let shown = self.page.take().filter(|page| page.folder == folder && offset == 0);
        let loaded = match shown {
            Some(shown) => self.merged_with(shown.loaded, total, envelopes),
            None => {
                self.relocated.clear();
                envelopes
            }
        };
        let page = ListPage::arranged(folder, offset, total, loaded, &self.arrangement, &self.folders);
        selection.page_arranged(folder, &page);
        self.remember_tags(selection.account.as_ref(), &page.loaded);
        self.clear_failure(selection.account.as_ref());
        self.page = Some(page);
    }

    /// The newest page put on top of the list that was shown. The messages
    /// named in `relocated` that are still below the page stay named, for a
    /// page that does not show the change yet; the others are settled.
    fn merged_with(&mut self, old: Vec<Envelope>, total: u32, new: Vec<Envelope>) -> Vec<Envelope> {
        let on_page: Vec<MessageId> = new.iter().map(|envelope| envelope.id).collect();
        let merged = merge_newest_without(old, total, new, &self.relocated);
        self.relocated
            .retain(|id| !on_page.contains(id) && merged.iter().any(|envelope| envelope.id == *id));
        merged
    }

    /// A page of older mail. It only means something next to the list it
    /// follows: the same folder, still open. The new messages take their
    /// places in the order the list is sorted in, and the selection stays on
    /// its message.
    pub(super) fn apply_older(&mut self, folder: FolderId, total: u32, envelopes: Vec<Envelope>, selection: &mut Selection) {
        self.remember_tags(selection.account.as_ref(), &envelopes);
        let Some(page) = self.page.as_mut().filter(|page| page.folder == folder) else {
            return;
        };
        if selection.folder != Some(folder) {
            return;
        }
        page.loaded = append_older(std::mem::take(&mut page.loaded), envelopes);
        page.total = total;
        page.arrange(&self.arrangement, &self.folders);
        selection.page_arranged(folder, page);
        self.clear_failure(selection.account.as_ref());
    }

    /// The server's new flags for messages: the loaded ones take them, and the
    /// rows are made again (a thread's row shows the marks of all its messages
    /// and a sort by flag moves a row).
    pub(super) fn apply_flags(&mut self, updates: Vec<(MessageId, Flags)>, selection: &mut Selection) {
        let Some(page) = self.page.as_mut() else {
            return;
        };
        for (id, flags) in updates {
            if let Some(envelope) = page.loaded.iter_mut().find(|envelope| envelope.id == id) {
                envelope.flags = flags;
            }
        }
        self.rearrange(selection);
    }

    /// Makes the rows of the page again after the arrangement or the loaded
    /// messages changed; the selection keeps its message.
    pub fn rearrange(&mut self, selection: &mut Selection) {
        let Some(page) = self.page.as_mut() else {
            return;
        };
        page.arrange(&self.arrangement, &self.folders);
        if selection.folder == Some(page.folder) {
            selection.page_arranged(page.folder, page);
        }
    }
}

#[cfg(test)]
include!("list_events.tests.rs");
