//! Dragging a bar with the pointer, and the pointer shape that invites it.

use surface::{CellPosition, PointerShape};

use super::drag::{Drag, DragSize, divider_at, size_for};
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
            None => false,
        }
    }

    /// Moves the bar being dragged to the pointer. Returns whether a drag is
    /// going, which means the screen needs drawing again.
    pub(crate) fn drag_to(&mut self, pos: CellPosition, grid: (u16, u16)) -> bool {
        let Some(drag) = self.drag else {
            return false;
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
        self.drag.take().is_some()
    }

    /// The pointer to show at a position: the resize pointer on a bar, and
    /// the one for the bar being dragged for as long as the drag lasts, even
    /// when the pointer has run ahead of it.
    pub fn pointer_shape(&self, col: u16, row: u16, grid: (u16, u16)) -> PointerShape {
        if let Some(drag) = self.drag {
            return drag.shape();
        }
        let layout = self.layout(grid.0, grid.1);
        divider_at(&layout, col, row).map_or(PointerShape::Arrow, Drag::shape)
    }
}
