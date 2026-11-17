//! `gg`: going to a folder by name. The key that starts the go-to prefix,
//! pressed twice, opens the folder picker the move key opens, for going to a
//! folder: it lists every folder of the account on screen (the open one and
//! the virtual views too), typed letters narrow the list, Enter opens the
//! highlighted folder with the focus in its list, Escape cancels. The picker's
//! keys and mouse are the move picker's (`picker`, `picker_mouse`); this file
//! is what is different about going to a folder.

use model::FolderId;

use crate::app_view::AppView;
use crate::move_picker::MovePicker;
use crate::view_state::Focus;

/// What the footer says when there is no folder to go to.
const NO_FOLDERS: &str = "No folders to go to yet";

impl AppView {
    /// Opens the picker for going to a folder. Nothing opens while the
    /// account on screen has no folders (none listed yet, or no account).
    pub(super) fn start_folder_picker(&mut self) {
        if self.mail.folders.is_empty() {
            self.mail.set_note(Some(NO_FOLDERS.to_string()));
            return;
        }
        self.picker = Some(MovePicker::for_opening());
    }

    /// Goes to the folder chosen in the picker as `g` and a letter goes to
    /// one: the folder opens as a click on it opens it, the folders above it
    /// open so it can be seen, and the focus is in its list, from whatever
    /// pane the picker was opened in. The folder that is open already stays
    /// as it is; it is only shown and given the focus.
    pub(super) fn open_picked_folder(&mut self, folder: FolderId) {
        self.account_cursor = None;
        self.focus = Focus::List;
        self.show_open_folder(folder);
    }
}

#[cfg(test)]
include!("folder_picker.tests.rs");
