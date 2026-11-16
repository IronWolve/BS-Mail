//! Dragging a bar with the pointer, and the pointer shape that invites it.

use surface::CellPosition;

use super::drag::{DragSize, divider_at, size_for};
use crate::app_view::AppView;

impl AppView {
    /// Starts a drag when the press is on a bar. Returns whether it was.
    pub(crate) fn begin_drag(&mut self, col: u16, row: u16, grid: (u16, u16)) -> bool {
        let layout = self.layout(grid.0, grid.1);
        match divider_at(&layout, col, row) {
            Some(drag) => {
                self.drag = Some(drag);
                true
            }
            None => self.press_scrollbar(col, row, grid),
        }
    }

    /// Moves the bar being dragged to the pointer. Returns whether a drag is
    /// going, which means the screen needs drawing again.
    pub(crate) fn drag_to(&mut self, pos: CellPosition, grid: (u16, u16)) -> bool {
        let Some(drag) = self.drag else {
            return self.drag_scrollbar(pos.row, grid);
        };
        let layout = self.layout(grid.0, grid.1);
        match size_for(drag, layout.main, pos.col, pos.row) {
            DragSize::SidebarWidth(width) => self.prefs.sidebar_width = width,
            DragSize::ListShare(share) => self.prefs.list_share = share,
        }
        true
    }

    /// Lets go of the bar. Returns whether one was held.
    pub(crate) fn end_drag(&mut self) -> bool {
        let bar = self.bar_drag.take().is_some();
        self.drag.take().is_some() || bar
    }
}
