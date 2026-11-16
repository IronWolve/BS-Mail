//! The wheel, and the views it moves. The wheel scrolls the pane under the
//! pointer and leaves the selection where it is; the view follows the
//! selection again only once a key moves it (see `ScrollState`).

use super::motion::folders_height;
use super::movement::WHEEL_ROWS;
use super::viewport::{folders_first, last_folders_first, list_first};
use crate::app_view::AppView;
use crate::layout::Layout;
use crate::widgets::{FolderRowInfo, row_offsets, visible_rows};

impl AppView {
    /// The first message row showing, for `visible` rows of `total`. Where the
    /// wheel or a scrollbar put the view while the selection is still the one
    /// it left; else wherever keeps the selected row in view.
    pub(crate) fn list_view_first(&self, visible: usize, total: usize) -> usize {
        let detached = self.scroll.list_detached.is_some() && self.scroll.list_detached == self.selection.message;
        if detached {
            return self.scroll.list.min(total.saturating_sub(visible));
        }
        list_first(self.scroll.list, self.selection.message_index, visible, total)
    }

    /// The first folder row showing; as `list_view_first`.
    pub(crate) fn folders_view_first(&self, rows: &[FolderRowInfo<'_>], offsets: &[u16], height: u16) -> usize {
        let detached = self.scroll.folders_detached.is_some() && self.scroll.folders_detached == self.selection.folder;
        if detached {
            return self.scroll.folders.min(last_folders_first(offsets, height));
        }
        folders_first(rows, offsets, self.selection.folder, self.scroll.folders, height)
    }

    /// One wheel step of `notches` (down is positive): the list's view moves
    /// by `WHEEL_ROWS` rows a notch, and the selected message stays.
    pub(super) fn wheel_list(&mut self, layout: &Layout, notches: i16) {
        let visible = layout.list.as_ref().map_or(0, |list| usize::from(list.rows.height));
        let total = self.shown_page().map_or(0, |page| page.envelopes.len());
        let first = self.list_view_first(visible, total);
        let to = first.saturating_add_signed(isize::from(notches) * WHEEL_ROWS as isize);
        self.show_list_from(to, visible, total);
    }

    /// Puts row `first` at the top of the list (or the last row that still
    /// fills it), leaving the selection alone.
    pub(super) fn show_list_from(&mut self, first: usize, visible: usize, total: usize) {
        let first = first.min(total.saturating_sub(visible));
        self.scroll.list = first;
        self.scroll.list_detached = self.selection.message;
        self.scroll.list_reach = (first + visible).saturating_sub(1);
    }

    /// The folder tree's view moves by `WHEEL_ROWS` rows a notch.
    pub(super) fn wheel_folders(&mut self, layout: &Layout, notches: i16) {
        let Some(spacing) = layout.sidebar.as_ref().map(|sidebar| sidebar.spacing) else {
            return;
        };
        let rows = visible_rows(&self.mail.folders);
        let offsets = row_offsets(&rows, spacing);
        let height = folders_height(layout);
        let first = self.folders_view_first(&rows, &offsets, height);
        let to = first.saturating_add_signed(isize::from(notches) * WHEEL_ROWS as isize);
        self.show_folders_from(to, &offsets, height);
    }

    /// Puts folder row `first` at the top of the tree (or the last row that
    /// still fills it), leaving the selection alone.
    pub(super) fn show_folders_from(&mut self, first: usize, offsets: &[u16], height: u16) {
        self.scroll.folders = first.min(last_folders_first(offsets, height));
        self.scroll.folders_detached = self.selection.folder;
    }
}

#[cfg(test)]
include!("wheel.tests.rs");
