//! The state of the path box and where its parts sit on a grid. The drawing
//! and the pointer both ask `path_parts`, so a click lands on what is seen.

use std::path::PathBuf;

use model::{AttachmentInfo, MessageId};
use surface::Rect;

use super::complete::{Match, matches};
use crate::composer::EditorBuffer;

/// What the path is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathFor {
    /// A file to attach to the message being written.
    Attach,
    /// Where to save an attachment of a message.
    SaveAs { message: MessageId, attachment: AttachmentInfo },
}

/// The most matches listed at once.
pub const MAX_ROWS: usize = 8;

/// The widest the box is drawn.
pub const WIDTH: u16 = 72;
/// The narrowest box that is drawn at all.
pub const MIN_WIDTH: u16 = 24;
/// Rows of the box without its list: border, context, the path, the reason, border.
pub const BASE_HEIGHT: u16 = 5;
/// What stands before the typed path.
pub const PROMPT: &str = "> ";
/// The words of the button that cancels.
pub const CANCEL_WORDS: &str = "Esc cancels";

#[derive(Debug, Clone)]
pub struct PathPrompt {
    pub purpose: PathFor,
    pub title: &'static str,
    /// The line above the typed path: what it is for.
    pub context: String,
    /// The words of the button that accepts: the key as a keycap and the word.
    pub go: &'static str,
    pub buffer: EditorBuffer,
    /// Why the last path was refused; the box stays open to correct it.
    pub error: Option<String>,
    /// The folders and files that match what is typed, listed under it.
    pub list: Vec<Match>,
    /// The row of the list that is highlighted, once the user moved to one.
    pub chosen: Option<usize>,
    /// The first row of the list that shows.
    pub first: usize,
    home: Option<PathBuf>,
}

impl PathPrompt {
    /// A box that asks for the file to attach, starting from `home`.
    pub fn attach(home: Option<PathBuf>) -> Self {
        Self::new(
            PathFor::Attach,
            " Attach a file ",
            "Type the path of a file, or pick one from the list (Tab completes, Down chooses)".to_owned(),
            "Enter attaches",
            home,
            "",
        )
    }

    /// A box that asks where to save `attachment`; it starts with the home
    /// folder, so a folder alone saves the file there under its own name.
    pub fn save_as(home: Option<PathBuf>, message: MessageId, attachment: AttachmentInfo, start: &str) -> Self {
        let context = format!("Save {} as: a folder saves it there under this name; a new file name saves it as that", attachment.name);
        Self::new(PathFor::SaveAs { message, attachment }, " Save attachment as ", context, "Enter saves", home, start)
    }

    fn new(purpose: PathFor, title: &'static str, context: String, go: &'static str, home: Option<PathBuf>, start: &str) -> Self {
        let mut prompt =
            Self { purpose, title, context, go, buffer: EditorBuffer::single_line(start), error: None, list: Vec::new(), chosen: None, first: 0, home };
        prompt.refresh();
        prompt
    }

    /// What was typed.
    pub fn typed(&self) -> String {
        self.buffer.text()
    }

    /// The home folder the box started from.
    pub fn home(&self) -> Option<&std::path::Path> {
        self.home.as_deref()
    }

    /// Lists the matches for what is typed again: after every change to it.
    pub fn refresh(&mut self) {
        self.list = matches(&self.typed(), self.home.as_deref());
        self.chosen = None;
        self.first = 0;
    }

    /// How many rows of the list show.
    pub fn visible_rows(&self) -> usize {
        self.list.len().min(MAX_ROWS)
    }

    /// Moves the highlight `by` rows (negative is up), keeping it in view.
    pub fn move_chosen(&mut self, by: isize) {
        if self.list.is_empty() {
            return;
        }
        let last = self.list.len() - 1;
        let next = match self.chosen {
            None if by < 0 => last,
            None => 0,
            Some(now) => now.saturating_add_signed(by).min(last),
        };
        self.chosen = Some(next);
        if next < self.first {
            self.first = next;
        } else if next >= self.first + MAX_ROWS {
            self.first = next + 1 - MAX_ROWS;
        }
    }

    /// Puts `text` in the box with the caret at its end, and lists its matches.
    pub fn set_typed(&mut self, text: &str) {
        self.buffer.set_text(text);
        self.error = None;
        self.refresh();
    }
}

/// Where the parts of the box are on a grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathParts {
    pub rect: Rect,
    pub context: Rect,
    /// The row inside the border that holds the typed path.
    pub line: Rect,
    /// The first cell of the typed path, after the prompt mark.
    pub text_x: u16,
    /// How many cells the typed path has before the right border.
    pub room: usize,
    pub error: Rect,
    /// The rows of the list that show.
    pub list: Rect,
    /// The words on the bottom border that accept, and that cancel.
    pub go: Rect,
    pub cancel: Rect,
}

fn cells(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

/// The parts of the box for `grid` with `rows` list rows, accepting with `go`,
/// or `None` when the grid is too small to draw it. The list gives way first
/// when there is little height.
pub fn path_parts(grid: Rect, go: &str, rows: usize) -> Option<PathParts> {
    let width = WIDTH.min(grid.width.saturating_sub(2));
    if width < MIN_WIDTH || grid.height < BASE_HEIGHT {
        return None;
    }
    let room_for_list = usize::from(grid.height.saturating_sub(BASE_HEIGHT + 1));
    let rows = u16::try_from(rows.min(room_for_list)).unwrap_or(0);
    let height = BASE_HEIGHT + rows;
    let top = grid.y + grid.height.saturating_sub(height) / 4;
    let rect = Rect::new(grid.x + (grid.width - width) / 2, top, width, height);
    let inner = |row: u16| Rect::new(rect.x + 1, rect.y + row, rect.width - 2, 1);
    let line = inner(2);
    let hint_cells = cells(go) + 3 + cells(CANCEL_WORDS);
    let bottom = rect.bottom() - 1;
    let go_at = (rect.right() - 1).saturating_sub(hint_cells + 1);
    Some(PathParts {
        rect,
        context: inner(1),
        line,
        text_x: line.x + 1 + cells(PROMPT),
        room: usize::from(line.width.saturating_sub(cells(PROMPT) + 2)),
        error: inner(3),
        list: Rect::new(rect.x + 1, rect.y + 4, rect.width - 2, rows),
        go: Rect::new(go_at, bottom, cells(go), 1),
        cancel: Rect::new(go_at + cells(go) + 3, bottom, cells(CANCEL_WORDS), 1),
    })
}

/// Which row of the list (counted from the first that shows) is at the cell.
pub fn list_row_at(parts: &PathParts, col: u16, row: u16) -> Option<usize> {
    let inside = col >= parts.list.x && col < parts.list.right() && row >= parts.list.y && row < parts.list.bottom();
    inside.then(|| usize::from(row - parts.list.y))
}

#[cfg(test)]
include!("prompt.tests.rs");
