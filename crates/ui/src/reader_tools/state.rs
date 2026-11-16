//! The reading pane's own state, in one place: the selected text, the find
//! box, the raw view and the pages for the browser. The screen holds one.

use model::{MessageId, RawPart};

use super::attachment_state::AttachmentState;
use super::find::FindState;
use super::raw_view::RawView;
use super::selection::TextSelection;

/// The safe copy of a message's HTML, ready for the program to write to a
/// file and open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlPage {
    pub html: String,
    /// How many references to the network were left out of it.
    pub removed: u32,
}

#[derive(Default)]
pub struct ReaderTools {
    pub selection: Option<TextSelection>,
    pub find: Option<FindState>,
    pub raw: Option<RawView>,
    /// Pages made and not yet taken by the program.
    pub pages: Vec<HtmlPage>,
    /// The message whose page was asked for and has not arrived.
    pub page_asked: Option<MessageId>,
    /// The address a press on the text was on: a release in the same place
    /// opens it, and a drag from it selects instead.
    pub pending_link: Option<String>,
    /// The attachments of the open message: the cursor, the downloads asked
    /// for and the files that arrived.
    pub attachments: AttachmentState,
}

impl ReaderTools {
    /// The raw view of the message `selected`, when that is what is open.
    pub fn raw_of(&self, selected: Option<MessageId>) -> Option<&RawView> {
        self.raw.as_ref().filter(|raw| Some(raw.message) == selected)
    }

    /// Lets go of what belonged to a message that is no longer the open one.
    pub fn forget_other_than(&mut self, selected: Option<MessageId>) {
        if self.raw.as_ref().is_some_and(|raw| Some(raw.message) != selected) {
            self.raw = None;
        }
    }

    /// The button that was held on the text is let go (its release never came,
    /// or something else began): a selection that spread stays, a press that
    /// never moved selects nothing, and the address it was on is forgotten.
    pub fn stop_dragging(&mut self) {
        self.pending_link = None;
        let Some(held) = self.selection.as_mut().filter(|held| held.dragging) else {
            return;
        };
        held.dragging = false;
        if !held.spread {
            self.selection = None;
        }
    }

    /// Asks for the raw text of `part` of `message`, which replaces whatever
    /// the pane showed of it.
    pub fn show_raw(&mut self, message: MessageId, part: RawPart) {
        self.raw = Some(RawView::waiting(message, part));
        self.selection = None;
    }
}

#[cfg(test)]
include!("state.tests.rs");
