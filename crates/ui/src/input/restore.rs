//! Coming back to where the user was: the account and folder remembered from
//! the last run are opened once the account list and the folder list say they
//! still exist. A place that is gone (an account removed, a folder deleted or
//! renamed) is dropped without a word, and the program starts where it always
//! did.

use model::{FolderId, Folder};

use crate::app_view::AppView;
use crate::view_state::Place;

impl AppView {
    /// Where the user is now, for keeping: the selected account's id and the
    /// open folder's path, each when known.
    pub fn place(&self) -> Place {
        let account = self.selection.account.as_ref().map(|id| id.0.clone());
        let folder = self.selection.folder.and_then(|id| self.mail.folder(id)).map(|folder| folder.path.clone());
        Place { account, folder }
    }

    /// Asks to open `place` as soon as what it names has been listed. Call it
    /// before the first events arrive, or after; it is tried at once and again
    /// as each account list and folder list comes in, until it is done or the
    /// place proves to be gone.
    pub fn restore_place(&mut self, place: Place) {
        self.restore = (place != Place::default()).then_some(place);
        self.apply_restore();
    }

    /// Takes the next step toward the remembered place, when it can: the
    /// account first (its folders come after the switch), then the folder.
    pub(crate) fn apply_restore(&mut self) {
        let Some(mut place) = self.restore.take() else {
            return;
        };
        if let Some(account) = place.account.clone() {
            if self.mail.accounts.is_empty() {
                self.restore = Some(place);
                return;
            }
            let Some(found) = self.mail.accounts.iter().find(|listed| listed.id.0 == account) else {
                return;
            };
            let id = found.id.clone();
            place.account = None;
            if self.switch_account(id) {
                // The new account's folders are still to come.
                self.restore = Some(place);
                return;
            }
        }
        let Some(path) = place.folder.clone() else {
            return;
        };
        if self.mail.folders.is_empty() {
            self.restore = Some(place);
            return;
        }
        let Some(id) = self.mail.folders.iter().find(|folder| folder.path == path).map(|folder| folder.id) else {
            return;
        };
        self.show_open_folder(id);
    }

    /// Opens folder `id` and opens the folders above it, so it can be seen.
    fn show_open_folder(&mut self, id: FolderId) {
        let mut parent = self.mail.folder(id).and_then(|folder| folder.parent);
        for _ in 0..self.mail.folders.len() {
            let Some(up) = parent else {
                break;
            };
            self.mail.expanded_by_user.insert(up, true);
            if let Some(folder) = self.mail.folders.iter_mut().find(|folder: &&mut Folder| folder.id == up) {
                folder.expanded = true;
                parent = folder.parent;
            } else {
                break;
            }
        }
        self.change_folder(id);
    }
}

#[cfg(test)]
include!("restore.tests.rs");
