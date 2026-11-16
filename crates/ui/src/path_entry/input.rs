//! Keys, text and the pointer in the path box. Typed text is the path and
//! every change lists the matches again; `Down` and `Up` choose a match, `Tab`
//! takes the chosen one (or the part all matches share), `Enter` takes a
//! chosen folder and goes into it, takes a chosen file, or accepts what was
//! typed, `Esc` cancels. A click on a match does what `Enter` does on it, on
//! the two words on the bottom border accepts and cancels, and outside the box
//! cancels. The box never touches a file: it only hands back the typed path.

use surface::{ButtonState, InputEvent, KeyEvent, LogicalKey, NamedKey, PointerButton, PointerShape, Rect};

use super::complete::common_start;
use super::prompt::{PathParts, PathPrompt, list_row_at, path_parts};
use crate::composer::pointer_moves::col_at_cell;
use crate::input::line_edit::edit_line;

/// What the box wants done after an input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathOutcome {
    /// Nothing for the box.
    Ignored,
    /// The box changed; draw it again.
    Changed,
    /// The user accepted this path.
    Accept(String),
    /// The user cancelled.
    Cancel,
}

/// How many rows `PageDown` and `PageUp` move.
const PAGE: isize = 8;

impl PathPrompt {
    /// A key press.
    pub fn key(&mut self, key: &KeyEvent) -> PathOutcome {
        let held = key.modifiers;
        if held.alt || held.meta {
            return PathOutcome::Ignored;
        }
        let LogicalKey::Named(named) = key.key else {
            return PathOutcome::Ignored;
        };
        match named {
            NamedKey::Escape => PathOutcome::Cancel,
            NamedKey::Enter => self.enter(),
            NamedKey::Tab => self.complete(),
            NamedKey::Down => self.moved(1),
            NamedKey::Up => self.moved(-1),
            NamedKey::PageDown => self.moved(PAGE),
            NamedKey::PageUp => self.moved(-PAGE),
            other => {
                self.error = None;
                let before = self.typed();
                if !edit_line(&mut self.buffer, other, held) {
                    return PathOutcome::Ignored;
                }
                if self.typed() != before {
                    self.refresh();
                }
                PathOutcome::Changed
            }
        }
    }

    /// Typed or pasted text.
    pub fn text(&mut self, text: &str) -> PathOutcome {
        let clean: String = text.chars().filter(|ch| !ch.is_control()).collect();
        if clean.is_empty() || !self.buffer.insert_text(&clean) {
            return PathOutcome::Ignored;
        }
        self.error = None;
        self.refresh();
        PathOutcome::Changed
    }

    fn moved(&mut self, by: isize) -> PathOutcome {
        self.move_chosen(by);
        PathOutcome::Changed
    }

    /// Enter: the chosen match, or what was typed.
    fn enter(&mut self) -> PathOutcome {
        match self.chosen {
            Some(at) => self.take(at, true),
            None => self.accept_typed(),
        }
    }

    fn accept_typed(&mut self) -> PathOutcome {
        let typed = self.typed();
        if typed.trim().is_empty() {
            self.error = Some("type the path of a file".to_owned());
            return PathOutcome::Changed;
        }
        PathOutcome::Accept(typed)
    }

    /// Takes match `at`: a folder goes into the box with its separator (and the
    /// list shows what is inside it); a file goes in too, and with `accept` is
    /// the answer.
    fn take(&mut self, at: usize, accept: bool) -> PathOutcome {
        let Some(found) = self.list.get(at).cloned() else {
            return PathOutcome::Ignored;
        };
        self.set_typed(&found.full);
        if accept && !found.is_dir { PathOutcome::Accept(found.full) } else { PathOutcome::Changed }
    }

    /// Tab: the chosen match; the only match; or the part every match shares.
    fn complete(&mut self) -> PathOutcome {
        if let Some(at) = self.chosen {
            return self.take(at, false);
        }
        if self.list.len() == 1 {
            return self.take(0, false);
        }
        let shared = common_start(&self.list);
        if shared.len() > self.typed().len() {
            self.set_typed(&shared);
        }
        PathOutcome::Changed
    }

    /// A pointer event while the box fills `grid`.
    pub fn pointer(&mut self, event: &InputEvent, grid: Rect) -> PathOutcome {
        let Some(parts) = path_parts(grid, self.go, self.visible_rows()) else {
            return PathOutcome::Ignored;
        };
        match event {
            InputEvent::Button { button: PointerButton::Left, state: ButtonState::Pressed, pos, modifiers } => {
                self.press(&parts, (pos.col, pos.row), modifiers.shift)
            }
            InputEvent::Wheel { rows, .. } if *rows != 0 => {
                let more = self.list.len().saturating_sub(super::prompt::MAX_ROWS);
                let next = self.first.saturating_add_signed(isize::from(*rows) * 3).min(more);
                let changed = next != self.first;
                self.first = next;
                if changed { PathOutcome::Changed } else { PathOutcome::Ignored }
            }
            _ => PathOutcome::Ignored,
        }
    }

    fn press(&mut self, parts: &PathParts, (col, row): (u16, u16), extend: bool) -> PathOutcome {
        let holds = |rect: Rect| col >= rect.x && col < rect.right() && row >= rect.y && row < rect.bottom();
        if holds(parts.go) {
            return self.accept_typed();
        }
        if holds(parts.cancel) || !holds(parts.rect) {
            return PathOutcome::Cancel;
        }
        if let Some(offset) = list_row_at(parts, col, row) {
            return self.take(self.first + offset, true);
        }
        if row == parts.line.y && col >= parts.text_x {
            let line = self.buffer.lines()[0].clone();
            let skip = crate::ask::name::text_skip(&line, self.buffer.cursor().col, parts.room);
            let target = col_at_cell(&line, skip, usize::from(col - parts.text_x));
            self.buffer.move_to_col(target, extend);
            return PathOutcome::Changed;
        }
        PathOutcome::Ignored
    }

    /// The pointer over the box at `at`: a hand over its words and the rows of
    /// the list, the text pointer over the path.
    pub fn pointer_shape(&self, grid: Rect, (col, row): (u16, u16)) -> PointerShape {
        let Some(parts) = path_parts(grid, self.go, self.visible_rows()) else {
            return PointerShape::Arrow;
        };
        let holds = |rect: Rect| col >= rect.x && col < rect.right() && row >= rect.y && row < rect.bottom();
        if holds(parts.go) || holds(parts.cancel) || list_row_at(&parts, col, row).is_some() {
            PointerShape::Hand
        } else if row == parts.line.y && col >= parts.text_x && col < parts.line.right() {
            PointerShape::Text
        } else {
            PointerShape::Arrow
        }
    }
}

#[cfg(test)]
include!("input.tests.rs");
