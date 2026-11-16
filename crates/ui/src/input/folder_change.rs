//! Changing the open folder: what the folder that was open leaves on screen
//! goes with it, so the list never shows one folder's mail under another's
//! name while the new folder's page is on its way.

use model::FolderId;

use crate::app_view::AppView;

impl AppView {
    /// The user chose `chosen` as the open folder. Nothing happens when it is
    /// the one open already.
    pub(crate) fn change_folder(&mut self, chosen: FolderId) {
        if self.selection.folder == Some(chosen) {
            return;
        }
        self.selection.folder = Some(chosen);
        self.forget_folder_view();
    }

    /// Drops the list, the message and its body, the marks, the open threads, the search and the
    /// scroll positions of the folder that was open, and the memory of what
    /// was asked for it, so its successor is asked for even when it was asked
    /// for before. The folder's own page, which the queue asks for next,
    /// fills them again.
    pub(crate) fn forget_folder_view(&mut self) {
        self.selection.message = None;
        self.selection.message_index = 0;
        self.search = None;
        self.scroll.reset_list();
        self.scroll.reader = 0;
        self.mail.page = None;
        self.mail.body = None;
        self.mail.relocated.clear();
        self.mail.arrangement.open.clear();
        self.marked.clear();
        self.queue.forget();
    }
}

#[cfg(test)]
include!("folder_change.tests.rs");
