//! The path box as one of the boxes over the mail: "Save as" for an
//! attachment. Typed text is the path; what is accepted goes to the engine as
//! a request for the attachment, and the program writes it where the path says
//! once it has arrived (a folder saves it there under its own name, a new file
//! name saves it as that). The box itself is `path_entry`.

use surface::{InputEvent, Rect};

use crate::app_view::AppView;
use crate::ask::Ask;
use crate::path_entry::{PathFor, PathOutcome, PathPrompt, home_dir};
use crate::reader_tools::{Owner, Want};

impl AppView {
    /// Opens the box that asks where to save an attachment of the open
    /// message: the one given, the cursor's, or the first.
    pub(crate) fn ask_save_as(&mut self, index: Option<usize>) {
        let Some((message, attachment, at)) = self.attachment_choice(index) else {
            self.mail.set_note(Some("This message has no attachments".to_owned()));
            return;
        };
        self.reading.attachments.cursor = Some((message, at));
        self.ask = Some(Ask::Path(PathPrompt::save_as(home_dir(), message, attachment, "~/")));
    }

    /// Input while the box is open. Returns whether the screen changed.
    pub(crate) fn path_ask_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        let Some(Ask::Path(prompt)) = self.ask.as_mut() else {
            return false;
        };
        let outcome = match event {
            InputEvent::Key(key) => prompt.key(key),
            InputEvent::Text(text) | InputEvent::Paste(text) => prompt.text(text),
            _ => prompt.pointer(event, Rect::new(0, 0, grid.0, grid.1)),
        };
        match outcome {
            PathOutcome::Ignored => false,
            PathOutcome::Changed => true,
            PathOutcome::Cancel => {
                self.ask = None;
                true
            }
            PathOutcome::Accept(typed) => {
                let Some(Ask::Path(prompt)) = self.ask.take() else {
                    return true;
                };
                if let PathFor::SaveAs { message, attachment } = prompt.purpose {
                    self.fetch_attachment(message, attachment, Owner::Reader(Want::SaveAs(typed)));
                }
                true
            }
        }
    }
}

#[cfg(test)]
include!("ask_path.tests.rs");
