//! Dragging message rows onto a folder: press on a row and move the pointer
//! off the cell, and the row (or the marked rows, when it is one of them) is
//! being dragged; the folder row under the pointer lights up while a drop
//! there would move them; letting go on it moves them with no question, as the
//! move picker does. Escape or any other key cancels. A folder that is not a
//! place to move to (the one they are in, or a search folder) does not light.

use model::{FolderId, FolderRole, MessageId};
use surface::CellPosition;

use super::hit::Hit;
use crate::app_view::AppView;
use crate::widgets::visible_rows;

/// The start of the footer's line while a drag is going.
const DRAGGING: &str = "Dragging ";

impl AppView {
    /// The pointer moved with the button held. Returns whether it is a drag
    /// of rows, which means the screen needs drawing again.
    pub(crate) fn drag_rows_to(&mut self, pos: CellPosition, grid: (u16, u16)) -> bool {
        let Some(drag) = self.memory.row_drag.as_ref() else {
            return false;
        };
        let cell = (pos.col, pos.row);
        if !drag.active && cell == drag.pressed {
            return false;
        }
        let (ids, started) = (drag.ids.clone(), !drag.active);
        let layout = self.layout(grid.0, grid.1);
        let over = self.drop_target(&layout, &ids, cell);
        if let Some(drag) = self.memory.row_drag.as_mut() {
            drag.active = true;
            drag.over = over;
        }
        if started {
            let what = if ids.len() == 1 { "1 message".to_string() } else { format!("{} messages", ids.len()) };
            self.mail.set_note(Some(format!("{DRAGGING}{what}: drop on a folder, Esc cancels")));
        }
        true
    }

    /// The button was let go: a drop on a folder row moves the rows. Returns
    /// whether a drag of rows was going.
    pub(crate) fn drop_rows(&mut self, pos: CellPosition, grid: (u16, u16)) -> bool {
        let Some(drag) = self.memory.row_drag.take() else {
            return false;
        };
        if !drag.active {
            return false;
        }
        self.clear_dragging_note();
        let layout = self.layout(grid.0, grid.1);
        let on_folder = matches!(self.hit(&layout, pos.col, pos.row), Hit::Folder(_));
        if on_folder && !self.changes_allowed_or_note() {
            return true;
        }
        if let Some(destination) = self.drop_target(&layout, &drag.ids, (pos.col, pos.row)) {
            let name = self.mail.folder(destination).map(|folder| folder.name.clone()).unwrap_or_default();
            let count = drag.ids.len();
            self.move_targets(drag.ids, destination);
            self.moved_note = Some(format!("Moved {count} to {name}"));
        }
        true
    }

    /// Ends a drag of rows without moving anything. Returns whether one was
    /// going.
    pub(crate) fn cancel_row_drag(&mut self) -> bool {
        let Some(drag) = self.memory.row_drag.take() else {
            return false;
        };
        if drag.active {
            self.clear_dragging_note();
        }
        drag.active
    }

    fn clear_dragging_note(&mut self) {
        if self.mail.error.as_deref().is_some_and(|note| note.starts_with(DRAGGING)) {
            self.mail.set_note(None);
        }
    }

    /// The folder a drop at `cell` would move `ids` to, when that is a place
    /// they can go.
    fn drop_target(&self, layout: &crate::layout::Layout, ids: &[MessageId], cell: (u16, u16)) -> Option<FolderId> {
        let Hit::Folder(index) = self.hit(layout, cell.0, cell.1) else {
            return None;
        };
        let folder = visible_rows(&self.mail.folders).get(index)?.folder;
        let virtual_folder = matches!(folder.role, FolderRole::Starred | FolderRole::Alerts);
        let same = Some(folder.id) == self.folder_all_are_in(ids);
        (!virtual_folder && !same && !self.mail.changes_refused).then_some(folder.id)
    }

    /// The folder a drop at `cell` would move the dragged rows to, for the
    /// pointer.
    pub(crate) fn drop_target_at(&self, cell: (u16, u16), grid: (u16, u16)) -> Option<FolderId> {
        let drag = self.memory.row_drag.as_ref().filter(|drag| drag.active)?;
        self.drop_target(&self.layout(grid.0, grid.1), &drag.ids, cell)
    }

    /// The folder a drop would move the dragged rows to now (where the pointer
    /// last was), for the highlight.
    pub(crate) fn drag_over(&self) -> Option<FolderId> {
        self.memory.row_drag.as_ref().filter(|drag| drag.active).and_then(|drag| drag.over)
    }

    /// Whether rows are being dragged.
    pub(crate) fn dragging_rows(&self) -> bool {
        self.memory.row_drag.as_ref().is_some_and(|drag| drag.active)
    }
}

#[cfg(test)]
include!("message_drag.tests.rs");
