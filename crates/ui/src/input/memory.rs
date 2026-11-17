//! What the last inputs left behind for the next one: where the pointer is,
//! the last row pressed (for a double-click and for a range), the rows being
//! dragged, and a `g` waiting for its letter.

use model::{FolderId, MessageId};

/// Longest gap, in milliseconds, between two presses on one row that makes a
/// double-click. Not configurable yet.
pub const DOUBLE_CLICK_MS: u64 = 400;

/// Rows picked up by a press on a message row, which become a drag once the
/// pointer leaves the cell it was pressed on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowDrag {
    /// The cell the button went down on.
    pub pressed: (u16, u16),
    /// The messages that would move: the marked ones when the row pressed is
    /// one of them, else the row alone.
    pub ids: Vec<MessageId>,
    /// The pointer has left the pressed cell: this is a drag now.
    pub active: bool,
    /// The folder the pointer is over and a drop would move the rows to.
    pub over: Option<FolderId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InputMemory {
    /// The cell the pointer was last over.
    pub hover: Option<(u16, u16)>,
    /// The message row last pressed and when (the caller's clock, in ms).
    pub last_press: Option<(MessageId, u64)>,
    /// The row a Shift+click marks the range from: the last one clicked.
    pub anchor: Option<MessageId>,
    /// Rows held for a drag onto a folder.
    pub row_drag: Option<RowDrag>,
    /// `g` was pressed and its letter is awaited.
    pub go_pending: bool,
    /// `F` was pressed and the letter of a folder action is awaited.
    pub folder_pending: bool,
    /// The note the footer was showing when the pointer came onto the link it
    /// is on: a note different from this one came after, and the footer keeps
    /// it over the hint about the link.
    pub hint_note: Option<String>,
}
