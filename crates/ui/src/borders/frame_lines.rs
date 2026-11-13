//! Picks the piece for a line cell from the directions its line continues,
//! so shared borders join: a corner, a tee or a cross in any border style.

use super::border_set::BorderSet;

/// The directions a line continues from one cell. The caller derives them
/// from the segments it draws: a cell on a horizontal rule that ends at a
/// frame has the frame's cell to its right, and that frame cell has the rule
/// to its left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Neighbors {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Neighbors {
    pub const fn new(up: bool, down: bool, left: bool, right: bool) -> Self {
        Self {
            up,
            down,
            left,
            right,
        }
    }

    /// How many directions the line continues in.
    pub fn count(self) -> usize {
        [self.up, self.down, self.left, self.right]
            .into_iter()
            .filter(|open| *open)
            .count()
    }
}

/// The piece for a cell whose line continues in the given directions. A
/// line that stops in one direction only still draws as a straight piece;
/// `None` when it continues nowhere.
pub fn junction_piece(set: &BorderSet, arms: Neighbors) -> Option<char> {
    let piece = match (arms.up, arms.down, arms.left, arms.right) {
        (false, false, false, false) => return None,
        (_, _, false, false) => set.vertical,
        (false, false, _, _) => set.horizontal,
        (false, true, false, true) => set.top_left,
        (false, true, true, false) => set.top_right,
        (true, false, false, true) => set.bottom_left,
        (true, false, true, false) => set.bottom_right,
        (false, true, true, true) => set.tee_down,
        (true, false, true, true) => set.tee_up,
        (true, true, false, true) => set.tee_right,
        (true, true, true, false) => set.tee_left,
        (true, true, true, true) => set.cross,
    };
    Some(piece)
}

#[cfg(test)]
include!("frame_lines.tests.rs");
