//! A grid of line segments: draw rules and frames into it, then ask each cell
//! for its piece. Where two lines meet, the cell gets a tee or a cross.

use super::border_set::BorderSet;
use super::frame_lines::{Neighbors, junction_piece};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineMap {
    width: u16,
    height: u16,
    arms: Vec<Neighbors>,
}

impl LineMap {
    pub fn new(width: u16, height: u16) -> Self {
        let cells = usize::from(width) * usize::from(height);
        Self {
            width,
            height,
            arms: vec![Neighbors::default(); cells],
        }
    }

    /// Adds a horizontal line on row `y` from `x_a` to `x_b`, both included
    /// and in either order. Cells outside the grid are ignored.
    pub fn horizontal(&mut self, y: u16, x_a: u16, x_b: u16) {
        let (from, to) = (x_a.min(x_b), x_a.max(x_b));
        for x in from..=to {
            if let Some(cell) = self.cell_mut(x, y) {
                cell.left |= x > from;
                cell.right |= x < to;
            }
        }
    }

    /// Adds a vertical line on column `x` from `y_a` to `y_b`, both included
    /// and in either order. Cells outside the grid are ignored.
    pub fn vertical(&mut self, x: u16, y_a: u16, y_b: u16) {
        let (from, to) = (y_a.min(y_b), y_a.max(y_b));
        for y in from..=to {
            if let Some(cell) = self.cell_mut(x, y) {
                cell.up |= y > from;
                cell.down |= y < to;
            }
        }
    }

    /// Adds the outline of a `width` x `height` box whose top-left cell is
    /// `(x, y)`. A box under two cells wide or high draws nothing.
    pub fn outline(&mut self, x: u16, y: u16, width: u16, height: u16) {
        if width < 2 || height < 2 {
            return;
        }
        let (right, bottom) = (x + width - 1, y + height - 1);
        self.horizontal(y, x, right);
        self.horizontal(bottom, x, right);
        self.vertical(x, y, bottom);
        self.vertical(right, y, bottom);
    }

    /// The directions the line continues from `(x, y)`; none outside the grid.
    pub fn arms_at(&self, x: u16, y: u16) -> Neighbors {
        self.index(x, y)
            .map(|i| self.arms[i])
            .unwrap_or_default()
    }

    /// The piece to draw at `(x, y)`, or `None` for an empty cell.
    pub fn piece_at(&self, set: &BorderSet, x: u16, y: u16) -> Option<char> {
        junction_piece(set, self.arms_at(x, y))
    }

    fn index(&self, x: u16, y: u16) -> Option<usize> {
        (x < self.width && y < self.height)
            .then(|| usize::from(y) * usize::from(self.width) + usize::from(x))
    }

    fn cell_mut(&mut self, x: u16, y: u16) -> Option<&mut Neighbors> {
        let index = self.index(x, y)?;
        self.arms.get_mut(index)
    }
}

#[cfg(test)]
include!("line_map.tests.rs");
