//! The footer's word that something the user asked for is done ("Made the
//! folder Projects", "Tagged 3 messages work"). It waits for the answer of the
//! backend that shows it: the new folder list, or the new tags. A failure
//! drops it, and the footer then says what went wrong instead. Once shown it
//! stays through the pages that follow (it is not a failure, which a page
//! ends) until the next key.

use model::Event;

use crate::app_view::AppView;

/// Which answer of the backend means the thing is done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Settles {
    /// The folder list is sent again (a folder made, renamed, deleted, or the
    /// counts of one that was marked read or emptied).
    Folders,
    /// The new tags of the messages.
    Tags,
}

/// The word waiting for its answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoneNote {
    text: String,
    settles: Settles,
}

impl AppView {
    /// Says `text` once the answer that `settles` it arrives.
    pub(crate) fn promise_note(&mut self, text: String, settles: Settles) {
        self.done_note = Some(DoneNote { text, settles });
    }

    /// What an event from the backend does to the word that waits: a failure
    /// drops it, and the answer it waits for returns it to be shown. Call
    /// before the event is applied, and show what it returns after.
    pub(crate) fn settle_done_note(&mut self, event: &Event) -> Option<String> {
        let settles = self.done_note.as_ref()?.settles;
        let arrives = match event {
            Event::OpFailed { .. } => {
                self.done_note = None;
                return None;
            }
            Event::FolderList { .. } => Settles::Folders,
            Event::TagsChanged { .. } => Settles::Tags,
            _ => return None,
        };
        if arrives == settles { self.done_note.take().map(|note| note.text) } else { None }
    }
}

#[cfg(test)]
include!("done_note.tests.rs");
