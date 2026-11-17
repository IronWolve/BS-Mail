//! The yes or no before a folder goes for good or an attachment that can carry
//! active content is opened (see `ask::confirm`): `y`
//! does it, `n` or Escape keeps things as they are, the arrows and Tab move
//! between the two buttons, Enter does what the focused button says, and a
//! click on a button does the same. A yes sends the command and says what was
//! done once the backend answers.

use model::Command;
use surface::{InputEvent, Rect};

use super::done_note::Settles;
use super::question::{Step, question_step};
use crate::app_view::AppView;
use crate::ask::{Ask, ConfirmAction};
use crate::delete_prompt::Choice;
use crate::reader_tools::{Owner, Want};

impl AppView {
    /// Input while the question is open. Returns whether the screen changed.
    pub(crate) fn confirm_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        let Some(Ask::Confirm(prompt)) = &self.ask else {
            return false;
        };
        let step = question_step(event, Rect::new(0, 0, grid.0, grid.1), (prompt.yes, prompt.no), prompt.focus);
        match step {
            Some(Step::Pick(choice)) => self.answer_confirm(choice),
            Some(Step::Toggle) => {
                if let Some(Ask::Confirm(prompt)) = self.ask.as_mut() {
                    prompt.toggle();
                }
            }
            None => return false,
        }
        true
    }

    fn answer_confirm(&mut self, choice: Choice) {
        let Some(Ask::Confirm(prompt)) = self.ask.take() else {
            return;
        };
        if choice == Choice::Keep {
            return;
        }
        let command = match prompt.action {
            ConfirmAction::DeleteFolder(folder) => Command::DeleteFolder { folder },
            ConfirmAction::EmptyFolder(folder) => Command::EmptyFolder { folder },
            ConfirmAction::OpenAttachment { message, attachment } => {
                // Asked for as `o` asks for it, once the question is answered: the program saves it and opens it.
                self.fetch_attachment(message, attachment, Owner::Reader(Want::Open));
                return;
            }
        };
        self.queue.push(command);
        self.promise_note(prompt.done, Settles::Folders);
    }
}

#[cfg(test)]
include!("ask_confirm.tests.rs");
