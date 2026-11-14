//! Opening and closing folders in the sidebar tree from the keyboard.

use model::FolderId;

use crate::app_view::AppView;
use crate::widgets::visible_rows;

/// What the selected folder is in the tree.
struct Place {
    id: FolderId,
    parent: Option<FolderId>,
    has_children: bool,
    expanded: bool,
}

impl AppView {
    fn selected_place(&self) -> Option<Place> {
        let selected = self.selection.folder?;
        visible_rows(&self.mail.folders)
            .into_iter()
            .find(|row| row.folder.id == selected)
            .map(|row| Place {
                id: row.folder.id,
                parent: row.folder.parent,
                has_children: row.has_children,
                expanded: row.folder.expanded,
            })
    }

    /// Opens or closes a folder and remembers the choice, so a fresh folder
    /// list from the backend keeps it.
    fn set_expanded(&mut self, id: FolderId, open: bool) {
        self.mail.expanded_by_user.insert(id, open);
        if let Some(folder) = self.mail.folders.iter_mut().find(|folder| folder.id == id) {
            folder.expanded = open;
        }
    }

    /// Right arrow in the folders: open a closed folder that has subfolders,
    /// otherwise go into the list.
    pub(crate) fn open_folder_or_go_in(&mut self) {
        match self.selected_place() {
            Some(place) if place.has_children && !place.expanded => self.set_expanded(place.id, true),
            _ => self.focus = crate::view_state::Focus::List,
        }
    }

    /// Left arrow in the folders: close an open folder, otherwise step up to
    /// its parent.
    pub(crate) fn close_folder_or_go_up(&mut self) {
        let Some(place) = self.selected_place() else {
            return;
        };
        if place.has_children && place.expanded {
            self.set_expanded(place.id, false);
        } else if let Some(parent) = place.parent {
            self.selection.folder = Some(parent);
            self.selection.message = None;
            self.selection.message_index = 0;
            self.search = None;
            self.scroll.list = 0;
            self.scroll.reader = 0;
        }
    }
}

#[cfg(test)]
include!("tree.tests.rs");
