//! Deleting mail for good: `d` in the Trash asks first. `y` deletes, `n` or
//! Escape keeps, the arrows and Tab move between the two buttons, Enter does
//! what the focused button says (and starts on Keep), and a click on a button
//! does the same. Nothing else reaches the screen underneath.

use surface::{ButtonState, InputEvent, KeyEvent, LogicalKey, NamedKey, PointerButton, Rect};

use crate::app_view::AppView;
use crate::delete_prompt::draw::{button_at, prompt_rects};
use crate::delete_prompt::{Choice, DeletePrompt};
use model::{Command, MessageId};

impl AppView {
    /// Whether `d` here would remove mail for good: the folder shown is the
    /// Trash.
    pub(crate) fn deleting_for_good(&self) -> bool {
        let Some(folder) = self.selection.folder else {
            return false;
        };
        self.mail.folders.iter().any(|candidate| candidate.id == folder && candidate.role == model::FolderRole::Trash)
    }

    /// Asks before `targets` are removed for good.
    pub(crate) fn ask_before_deleting(&mut self, targets: Vec<MessageId>) {
        self.delete_prompt = Some(DeletePrompt::new(targets));
    }

    /// Input while the question is open. Returns whether the screen changed.
    pub(crate) fn delete_prompt_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        match event {
            InputEvent::Key(key) => self.delete_prompt_key(key),
            InputEvent::Button { button: PointerButton::Left, state: ButtonState::Pressed, pos, .. } => {
                let area = Rect::new(0, 0, grid.0, grid.1);
                let choice = prompt_rects(area).and_then(|rects| button_at(&rects, pos.col, pos.row));
                match choice {
                    Some(choice) => self.answer_delete_prompt(choice),
                    None => return false,
                }
                true
            }
            _ => false,
        }
    }

    fn delete_prompt_key(&mut self, key: &KeyEvent) -> bool {
        if key.modifiers.ctrl || key.modifiers.alt || key.modifiers.meta {
            return false;
        }
        let focus = self.delete_prompt.as_ref().map_or(Choice::Keep, |prompt| prompt.focus);
        match key.key {
            LogicalKey::Char('y' | 'Y') => self.answer_delete_prompt(Choice::Delete),
            LogicalKey::Char('n' | 'N') | LogicalKey::Named(NamedKey::Escape) => self.answer_delete_prompt(Choice::Keep),
            LogicalKey::Named(NamedKey::Enter) => self.answer_delete_prompt(focus),
            LogicalKey::Named(NamedKey::Left | NamedKey::Right | NamedKey::Tab) => {
                if let Some(prompt) = self.delete_prompt.as_mut() {
                    prompt.toggle();
                }
            }
            _ => return false,
        }
        true
    }

    fn answer_delete_prompt(&mut self, choice: Choice) {
        let Some(prompt) = self.delete_prompt.take() else {
            return;
        };
        if choice == Choice::Keep {
            return;
        }
        self.send_delete(prompt.targets);
    }

    /// Removes the messages and moves the selection to the nearest one that
    /// stays, so the refreshed page keeps the user's place.
    pub(crate) fn send_delete(&mut self, ids: Vec<MessageId>) {
        let neighbor = self.neighbor_outside(&ids);
        self.selection.message = neighbor;
        self.clear_marks();
        self.queue.push(Command::Delete { messages: ids });
    }
}

#[cfg(test)]
include!("delete_prompt.tests.rs");
