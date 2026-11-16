//! The box that asks for a folder's name (see `ask::name`). Typed text is the
//! name; Enter checks it and asks the backend, Escape cancels. A name that
//! cannot work is refused in words under the line and the box stays open to be
//! corrected. The mouse works in the box: a click in the name puts the caret
//! under the pointer (with Shift it selects), the two words on the bottom
//! border accept and cancel, and a click outside the box cancels.

use model::{Command, check_folder_name};
use surface::{ButtonState, InputEvent, KeyEvent, LogicalKey, NamedKey, PointerButton, PointerShape, Rect};

use super::done_note::Settles;
use super::line_edit::edit_line;
use crate::app_view::AppView;
use crate::ask::{Ask, NameFor, NamePrompt};
use crate::ask::name::{holds, name_parts, text_skip};
use crate::composer::pointer_moves::col_at_cell;

impl AppView {
    /// Input while the box is open. Returns whether the screen changed.
    pub(crate) fn name_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        match event {
            InputEvent::Key(key) => self.name_key(key),
            InputEvent::Text(text) | InputEvent::Paste(text) => {
                let clean: String = text.chars().filter(|ch| !ch.is_control()).collect();
                self.name_prompt_mut().is_some_and(|prompt| {
                    prompt.error = None;
                    prompt.buffer.insert_text(&clean)
                })
            }
            _ => self.name_pointer(event, grid),
        }
    }

    fn name_prompt_mut(&mut self) -> Option<&mut NamePrompt> {
        match self.ask.as_mut() {
            Some(Ask::Name(prompt)) => Some(prompt),
            _ => None,
        }
    }

    fn name_key(&mut self, key: &KeyEvent) -> bool {
        let held = key.modifiers;
        let LogicalKey::Named(named) = key.key else {
            return false;
        };
        let Some(prompt) = self.name_prompt_mut() else {
            return false;
        };
        match named {
            NamedKey::Escape => self.ask = None,
            NamedKey::Enter => self.finish_name(),
            other => {
                prompt.error = None;
                return edit_line(&mut prompt.buffer, other, held);
            }
        }
        true
    }

    /// Checks the typed name and asks for the folder; a refusal stays in the
    /// box.
    pub(crate) fn finish_name(&mut self) {
        let Some(Ask::Name(prompt)) = &self.ask else {
            return;
        };
        let name = match check_folder_name(&prompt.typed()).and_then(|name| self.name_is_free(prompt, name)) {
            Ok(name) => name,
            Err(reason) => {
                if let Some(prompt) = self.name_prompt_mut() {
                    prompt.error = Some(reason);
                }
                return;
            }
        };
        let Some(Ask::Name(prompt)) = self.ask.take() else {
            return;
        };
        let command = match (prompt.purpose, self.selection.account.clone()) {
            (NameFor::NewFolder { inside }, Some(account)) => Command::CreateFolder { account, parent: inside, name: name.clone() },
            (NameFor::Rename { folder }, _) => Command::RenameFolder { folder, name: name.clone() },
            (NameFor::NewFolder { .. }, None) => return,
        };
        self.queue.push(command);
        self.promise_note((prompt.done)(&name), Settles::Folders);
    }

    /// The name, unless a folder next to the new one already has it, which
    /// the screen can see without asking the server.
    fn name_is_free(&self, prompt: &NamePrompt, name: String) -> Result<String, String> {
        let (parent, own) = match prompt.purpose {
            NameFor::NewFolder { inside } => (inside, None),
            NameFor::Rename { folder } => (self.mail.folder(folder).and_then(|found| found.parent), Some(folder)),
        };
        let taken = self.mail.folders.iter().any(|other| {
            other.parent == parent && Some(other.id) != own && !other.role.is_virtual() && other.name.eq_ignore_ascii_case(&name)
        });
        match (taken, own.and_then(|id| self.mail.folder(id))) {
            (_, Some(found)) if found.name == name => Err("that is already its name".to_owned()),
            (true, _) => Err(format!("a folder named {name} is already there")),
            _ => Ok(name),
        }
    }

    /// A pointer event while the box is open.
    fn name_pointer(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        let InputEvent::Button { button: PointerButton::Left, state: ButtonState::Pressed, pos, modifiers } = event else {
            return false;
        };
        let go = match &self.ask {
            Some(Ask::Name(prompt)) => prompt.go,
            _ => return false,
        };
        let Some(parts) = name_parts(Rect::new(0, 0, grid.0, grid.1), go) else {
            return false;
        };
        let (col, row) = (pos.col, pos.row);
        if holds(parts.go, col, row) {
            self.finish_name();
        } else if holds(parts.cancel, col, row) || !holds(parts.rect, col, row) {
            self.ask = None;
        } else if row == parts.line.y
            && col >= parts.text_x
            && let Some(prompt) = self.name_prompt_mut()
        {
            let line = prompt.buffer.lines()[0].clone();
            let skip = text_skip(&line, prompt.buffer.cursor().col, parts.room);
            let target = col_at_cell(&line, skip, usize::from(col - parts.text_x));
            prompt.buffer.move_to_col(target, modifiers.shift);
        }
        true
    }

    /// The pointer over the open box: a hand over its two words, the text
    /// pointer over the name.
    pub(crate) fn name_pointer_shape(&self, col: u16, row: u16, grid: (u16, u16)) -> PointerShape {
        let Some(Ask::Name(prompt)) = &self.ask else {
            return PointerShape::Arrow;
        };
        let Some(parts) = name_parts(Rect::new(0, 0, grid.0, grid.1), prompt.go) else {
            return PointerShape::Arrow;
        };
        if holds(parts.go, col, row) || holds(parts.cancel, col, row) {
            PointerShape::Hand
        } else if row == parts.line.y && col >= parts.text_x && col < parts.line.right() {
            PointerShape::Text
        } else {
            PointerShape::Arrow
        }
    }
}

#[cfg(test)]
include!("ask_name.tests.rs");
