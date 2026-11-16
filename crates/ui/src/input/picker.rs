//! Moving messages to another folder with the picker.
//!
//! `m` opens it for the marked messages, or the selected one. Typed letters
//! narrow the folders, the arrows move the highlight, Enter moves and Escape
//! cancels. It only asks the backend; the list changes when the backend says
//! the messages have moved.

use model::{Command, FolderId, MessageId};
use surface::{InputEvent, KeyEvent, LogicalKey, NamedKey};

use crate::app_view::AppView;
use crate::move_picker::MovePicker;

const READ_ONLY_NOTE: &str = "this account is read-only: set read_only = false in its file to change mail";

impl AppView {
    pub(crate) fn start_move(&mut self) {
        let targets = self.targets();
        if targets.is_empty() {
            return;
        }
        if self.mail.changes_refused {
            self.mail.set_note(Some(READ_ONLY_NOTE.to_string()));
            return;
        }
        let from = self.folder_all_are_in(&targets);
        self.picker = Some(MovePicker::new(targets, from));
    }

    /// The folder every one of `ids` is in, when there is one: the only
    /// folder that is no place to move them to. A list that is the answer to
    /// a search holds mail from several folders, so the folder shown says
    /// nothing about where a message is; its own folder does. Mail whose
    /// folder cannot be told is taken to be in the folder shown.
    pub(crate) fn folder_all_are_in(&self, ids: &[MessageId]) -> Option<FolderId> {
        let shown = self.mail.page.as_ref().map(|page| page.envelopes.as_slice()).unwrap_or_default();
        let mut folders = shown.iter().filter(|envelope| ids.contains(&envelope.id)).map(|envelope| envelope.folder);
        let Some(first) = folders.next() else {
            return self.selection.folder;
        };
        folders.all(|folder| folder == first).then_some(first)
    }

    /// Input while the picker is open. Everything is the picker's: nothing
    /// reaches the screen underneath. Returns whether the screen changed.
    pub(crate) fn picker_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        match event {
            InputEvent::Key(key) => self.picker_key(key),
            InputEvent::Text(text) | InputEvent::Paste(text) => {
                if let Some(picker) = self.picker.as_mut() {
                    picker.type_text(text);
                }
                true
            }
            _ => self.picker_pointer(event, grid),
        }
    }

    fn picker_key(&mut self, key: &KeyEvent) -> bool {
        let LogicalKey::Named(named) = key.key else {
            return false;
        };
        let shown = self.picker.as_ref().map_or(0, |picker| picker.candidates(&self.mail.folders).len());
        let Some(picker) = self.picker.as_mut() else {
            return false;
        };
        match named {
            NamedKey::Escape => self.picker = None,
            NamedKey::Up => picker.step(false, shown),
            NamedKey::Down => picker.step(true, shown),
            NamedKey::PageUp => (0..8).for_each(|_| picker.step(false, shown)),
            NamedKey::PageDown => (0..8).for_each(|_| picker.step(true, shown)),
            NamedKey::Backspace => picker.backspace(),
            NamedKey::Enter => self.finish_move(),
            _ => return false,
        }
        true
    }

    /// Asks for the move to the highlighted folder, and moves the selection
    /// to the nearest message that stays so the user keeps their place.
    pub(crate) fn finish_move(&mut self) {
        let Some(picker) = self.picker.take() else {
            return;
        };
        let Some(destination) = picker.chosen(&self.mail.folders).map(|folder| folder.id) else {
            // Nothing matches: the picker stays open to be corrected.
            self.picker = Some(picker);
            return;
        };
        self.move_targets(picker.targets, destination);
    }

    /// Asks for `targets` to move to `destination`, with the selection moved to
    /// the nearest message that stays, the marks cleared and the rows hidden
    /// until the backend says they moved.
    pub(crate) fn move_targets(&mut self, targets: Vec<MessageId>, destination: FolderId) {
        self.select_neighbor_outside(&targets);
        self.clear_marks();
        self.mail.relocated.extend(&targets);
        self.queue.push(Command::Move {
            messages: targets,
            to: destination,
        });
    }
}

#[cfg(test)]
include!("picker.tests.rs");
