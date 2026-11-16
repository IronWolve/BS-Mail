//! The box that asks for a folder's name: a new folder (inside one, or at the
//! top) or a new name for one. One line to type in, and the reason a name was
//! refused under it.

use model::FolderId;
use surface::Rect;

use crate::composer::EditorBuffer;
use crate::composer::graphemes::{cells_between, count};

/// What the name is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameFor {
    NewFolder { inside: Option<FolderId> },
    Rename { folder: FolderId },
}

#[derive(Debug, Clone)]
pub struct NamePrompt {
    pub purpose: NameFor,
    pub title: &'static str,
    /// The line above the typed name: where the folder goes, or which folder
    /// is renamed.
    pub context: String,
    /// The words of the button that accepts, as they stand on the bottom
    /// border: the key as a keycap and the word after it.
    pub go: &'static str,
    pub buffer: EditorBuffer,
    /// Why the last name was refused; the box stays open to correct it.
    pub error: Option<String>,
    /// What the footer says once the server has done it, built from the name
    /// when it is accepted.
    pub done: fn(&str) -> String,
}

fn made(name: &str) -> String {
    format!("Made the folder {name}")
}

fn renamed(name: &str) -> String {
    format!("Renamed the folder to {name}")
}

impl NamePrompt {
    /// A new folder inside `inside` (its name), or at the top.
    pub fn new_folder(inside: Option<(FolderId, &str)>) -> Self {
        let (context, inside) = match inside {
            Some((id, name)) => (format!("Inside {name}"), Some(id)),
            None => ("At the top level".to_owned(), None),
        };
        Self {
            purpose: NameFor::NewFolder { inside },
            title: " New folder ",
            context,
            go: "Enter makes it",
            buffer: EditorBuffer::single_line(""),
            error: None,
            done: made,
        }
    }

    /// A new name for `folder`, which is called `current` now.
    pub fn rename(folder: FolderId, current: &str) -> Self {
        Self {
            purpose: NameFor::Rename { folder },
            title: " Rename folder ",
            context: format!("Rename {current} to"),
            go: "Enter renames",
            buffer: EditorBuffer::single_line(current),
            error: None,
            done: renamed,
        }
    }

    /// What was typed.
    pub fn typed(&self) -> String {
        self.buffer.text()
    }
}

/// The widest the box is drawn.
pub const WIDTH: u16 = 72;
/// The narrowest box that is drawn at all.
pub const MIN_WIDTH: u16 = 20;
/// The box is this many rows: border, context, the name, the reason, border.
pub const HEIGHT: u16 = 5;
/// What stands before the typed name.
pub const PROMPT: &str = "> ";
/// The words of the button that cancels.
pub const CANCEL_WORDS: &str = "Esc cancels";

fn cells(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

/// Where the parts of the box are on a grid: the drawing and the pointer both
/// ask here, so a click lands on what is seen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameParts {
    pub rect: Rect,
    pub context: Rect,
    /// The row inside the border that holds the typed name.
    pub line: Rect,
    /// The first cell of the typed name, after the prompt mark.
    pub text_x: u16,
    /// How many cells the typed name has before the right border.
    pub room: usize,
    pub error: Rect,
    /// The words on the bottom border that accept, and that cancel.
    pub go: Rect,
    pub cancel: Rect,
}

/// The parts of the box for `grid`, accepting with `go`, or `None` when the
/// grid is too small to draw it.
pub fn name_parts(grid: Rect, go: &str) -> Option<NameParts> {
    let width = WIDTH.min(grid.width.saturating_sub(2));
    if width < MIN_WIDTH || grid.height < HEIGHT {
        return None;
    }
    let rect = Rect::new(grid.x + (grid.width - width) / 2, grid.y + grid.height.saturating_sub(HEIGHT) / 4, width, HEIGHT);
    let inner = |row: u16| Rect::new(rect.x + 1, rect.y + row, rect.width - 2, 1);
    let line = inner(2);
    let hint_cells = cells(go) + 3 + cells(CANCEL_WORDS);
    let bottom = rect.bottom() - 1;
    let go_at = (rect.right() - 1).saturating_sub(hint_cells + 1);
    Some(NameParts {
        rect,
        context: inner(1),
        line,
        text_x: line.x + 1 + cells(PROMPT),
        room: usize::from(line.width.saturating_sub(cells(PROMPT) + 2)),
        error: inner(3),
        go: Rect::new(go_at, bottom, cells(go), 1),
        cancel: Rect::new(go_at + cells(go) + 3, bottom, cells(CANCEL_WORDS), 1),
    })
}

/// How many graphemes of the name scroll off to the left to keep the caret at
/// grapheme `caret` in view in `room` cells.
pub fn text_skip(line: &str, caret: usize, room: usize) -> usize {
    let mut skip = 0;
    while cells_between(line, skip, caret) >= room && skip < count(line) {
        skip += 1;
    }
    skip
}

/// Whether the cell is inside `rect`.
pub fn holds(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.right() && row >= rect.y && row < rect.bottom()
}

#[cfg(test)]
include!("name.tests.rs");
