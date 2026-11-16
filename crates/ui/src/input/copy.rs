//! Copying text out of the program: `y` copies the text selected in the
//! reader, or the whole message when nothing is selected, and the subject in
//! the list; `Y` copies the sender's address. The text goes to the clipboard
//! through the host (see `take_copied`), and the footer says how much.

use crate::app_view::AppView;
use crate::view_state::Focus;

/// The most characters one copy puts on the clipboard: a terminal's clipboard
/// escape carries a limited text, and a message can be megabytes.
pub const COPY_LIMIT: usize = 1_000_000;

fn characters(count: usize) -> String {
    if count == 1 { "1 character".to_owned() } else { format!("{count} characters") }
}

impl AppView {
    /// Puts `text` on the clipboard, for the host to take, and says so in the
    /// footer: `Copied 214 characters`, or `Copied the subject (31 characters)`
    /// for `what`. Nothing to copy says so too.
    pub(crate) fn put_on_clipboard(&mut self, what: Option<&str>, text: String) {
        if text.is_empty() {
            self.mail.set_note(Some("There is nothing to copy".to_owned()));
            return;
        }
        let total = text.chars().count();
        let text: String = text.chars().take(COPY_LIMIT).collect();
        let count = text.chars().count();
        let mut note = match what {
            Some(what) => format!("Copied {what} ({})", characters(count)),
            None => format!("Copied {}", characters(count)),
        };
        if count < total {
            note.push_str(&format!(", the first of {total}"));
        }
        self.mail.set_note(Some(note));
        self.copied_link = Some(text);
    }

    /// `y`: in the list the subject, elsewhere the selected text of the
    /// reader, or the whole message when none is selected.
    pub(crate) fn copy_key(&mut self, grid: (u16, u16)) {
        if self.focus == Focus::List {
            self.copy_subject();
        } else {
            self.copy_message_text(grid);
        }
    }

    /// The selected text of the reader, else every line of the message.
    pub(crate) fn copy_message_text(&mut self, grid: (u16, u16)) {
        let layout = self.layout(grid.0, grid.1);
        let selected = self.reader_text(&layout).and_then(|(text, _)| {
            let (start, end) = self.reading.selection.as_ref()?.range(text.key())?;
            Some(text.text_between(start, end))
        });
        match selected.or_else(|| self.whole_text()) {
            Some(text) => self.put_on_clipboard(None, text),
            None => self.mail.set_note(Some("The message has not arrived yet, so there is nothing to copy".to_owned())),
        }
    }
}

#[cfg(test)]
include!("copy.tests.rs");
