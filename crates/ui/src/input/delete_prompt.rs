//! Deleting mail: `d` and the Delete key (and the Delete button) always ask
//! first, for one message or for many, whether the mail moves to the Trash or,
//! in the Trash, goes for good. `y` confirms, `n` or Escape keeps, the arrows
//! and Tab move between the two buttons, Enter does what the focused button
//! says (and starts on Keep), and a click on a button does the same. Nothing
//! else reaches the screen underneath.

use surface::{InputEvent, Rect};

use super::question::{Step, question_step, refuses};
use crate::app_view::AppView;
use crate::delete_prompt::draw::confirm_label;
use crate::delete_prompt::{Choice, DeletePrompt, draw::KEEP};
use model::{Command, MessageId};

impl AppView {
    /// How many of `ids` a delete would remove for good: the ones whose own folder
    /// is the Trash. The folder shown says nothing about it when the list is
    /// the answer to a search, which holds mail from every folder. Mail whose
    /// folder cannot be told is judged by the folder shown, the careful way.
    pub(crate) fn count_deleted_for_good(&self, ids: &[MessageId]) -> usize {
        let is_trash = |folder| {
            self.mail.folders.iter().any(|candidate| candidate.id == folder && candidate.role == model::FolderRole::Trash)
        };
        let shown = self.mail.page.as_ref().map(|page| page.envelopes.as_slice()).unwrap_or_default();
        let known: Vec<_> = shown.iter().filter(|envelope| ids.contains(&envelope.id)).collect();
        let unknown = ids.len() - known.len().min(ids.len());
        let in_trash = known.iter().filter(|envelope| is_trash(envelope.folder)).count();
        let unknown_in_trash = if self.selection.folder.is_some_and(is_trash) { unknown } else { 0 };
        in_trash + unknown_in_trash
    }

    /// Asks before `targets` are deleted, `for_good` of them for good and the
    /// rest to the Trash.
    pub(crate) fn ask_before_deleting(&mut self, targets: Vec<MessageId>, for_good: usize) {
        self.delete_prompt = Some(DeletePrompt::counting(targets, for_good));
    }

    /// Input while the question is open. Returns whether the screen changed.
    pub(crate) fn delete_prompt_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        if self.layout(grid.0, grid.1).too_small {
            // The screen then says only that the window is too small, so the
            // question cannot be seen: it can be dismissed but never answered
            // yes, whatever key is pressed.
            return match event {
                InputEvent::Key(key) if refuses(key) => {
                    self.answer_delete_prompt(Choice::Keep);
                    true
                }
                _ => false,
            };
        }
        let Some(prompt) = self.delete_prompt.as_ref() else {
            return false;
        };
        let labels = (confirm_label(prompt.for_good), KEEP);
        match question_step(event, Rect::new(0, 0, grid.0, grid.1), labels, prompt.focus) {
            Some(Step::Pick(choice)) => self.answer_delete_prompt(choice),
            Some(Step::Toggle) => {
                if let Some(prompt) = self.delete_prompt.as_mut() {
                    prompt.toggle();
                }
            }
            None => return false,
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
        self.select_neighbor_outside(&ids);
        self.clear_marks();
        self.mail.relocated.extend(&ids);
        self.queue.push(Command::Delete { messages: ids });
    }
}

#[cfg(test)]
include!("delete_prompt.tests.rs");
