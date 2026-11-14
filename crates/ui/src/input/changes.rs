//! Changing mail from the screen: read and unread, the star, delete, and
//! marking a message read once it has been open long enough.
//!
//! Each of these only asks the backend. The screen changes when the backend
//! says it happened, so a refusal or a failure never leaves it showing
//! something the server does not have.

use model::{Command, Envelope, FlagChange, MessageId};

use crate::app_view::AppView;

/// What the footer says when the account does not allow changes.
const READ_ONLY_NOTE: &str = "this account is read-only: set read_only = false in its file to change mail";

impl AppView {
    /// The envelope of the selected message, when its folder's page is shown.
    fn current_envelope(&self) -> Option<&Envelope> {
        let folder = self.selection.folder?;
        let page = self.mail.page.as_ref().filter(|page| page.folder == folder)?;
        page.envelopes.get(self.selection.message_index)
    }

    /// Whether a change may be sent; when it may not, says why in the footer.
    fn changes_allowed_or_note(&mut self) -> bool {
        if self.mail.changes_refused {
            self.mail.error = Some(READ_ONLY_NOTE.to_string());
            return false;
        }
        true
    }

    pub(crate) fn toggle_read(&mut self) {
        let Some((id, seen)) = self.current_envelope().map(|e| (e.id, e.flags.seen)) else {
            return;
        };
        if !self.changes_allowed_or_note() {
            return;
        }
        // The user's own choice wins over the timer for this message.
        self.auto_marked = Some(id);
        self.read_timer = None;
        self.queue.push(set_flags(id, FlagChange { seen: Some(!seen), ..FlagChange::default() }));
    }

    pub(crate) fn toggle_star(&mut self) {
        let Some((id, flagged)) = self.current_envelope().map(|e| (e.id, e.flags.flagged)) else {
            return;
        };
        if !self.changes_allowed_or_note() {
            return;
        }
        self.queue.push(set_flags(id, FlagChange { flagged: Some(!flagged), ..FlagChange::default() }));
    }

    /// Deletes the selected message and moves the selection to its neighbor,
    /// so the refreshed page keeps the user's place.
    pub(crate) fn delete_selected(&mut self) {
        let Some(id) = self.current_envelope().map(|e| e.id) else {
            return;
        };
        if !self.changes_allowed_or_note() {
            return;
        }
        let neighbor = self.neighbor_of_selected();
        self.selection.message = neighbor;
        self.queue.push(Command::Delete { messages: vec![id] });
    }

    /// The message after the selected one, else the one before it.
    fn neighbor_of_selected(&self) -> Option<MessageId> {
        let folder = self.selection.folder?;
        let page = self.mail.page.as_ref().filter(|page| page.folder == folder)?;
        let at = self.selection.message_index;
        page.envelopes
            .get(at + 1)
            .or_else(|| at.checked_sub(1).and_then(|before| page.envelopes.get(before)))
            .map(|envelope| envelope.id)
    }

    /// Marks the selected message read once its body has been showing for
    /// `reader.mark_read_after_ms`. `now_ms` is any steady clock in
    /// milliseconds; the caller calls this a few times a second.
    pub fn tick(&mut self, now_ms: u64) {
        let Some(id) = self.selection.message else {
            self.read_timer = None;
            return;
        };
        if self.auto_marked.is_some_and(|marked| marked != id) {
            self.auto_marked = None;
        }
        let showing = self.mail.body.as_ref().is_some_and(|(shown, _)| *shown == id);
        let unseen = self.current_envelope().is_some_and(|e| e.id == id && !e.flags.seen);
        if !showing || !unseen || self.auto_marked == Some(id) || self.mail.changes_refused {
            self.read_timer = None;
            return;
        }
        let due = match self.read_timer {
            Some((timed, due)) if timed == id => due,
            _ => {
                let due = now_ms.saturating_add(u64::from(self.config.reader.mark_read_after_ms));
                self.read_timer = Some((id, due));
                due
            }
        };
        if now_ms >= due {
            self.read_timer = None;
            self.auto_marked = Some(id);
            self.queue.push(set_flags(id, FlagChange { seen: Some(true), ..FlagChange::default() }));
        }
    }
}

fn set_flags(id: MessageId, change: FlagChange) -> Command {
    Command::SetFlags {
        messages: vec![id],
        change,
    }
}

#[cfg(test)]
include!("changes.tests.rs");
