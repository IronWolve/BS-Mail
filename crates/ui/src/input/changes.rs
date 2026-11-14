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

    /// The envelopes of the messages a change acts on.
    fn target_envelopes(&self) -> Vec<&Envelope> {
        let ids = self.targets();
        let folder = self.selection.folder;
        let Some(page) = folder.and_then(|f| self.mail.page.as_ref().filter(|p| p.folder == f)) else {
            return Vec::new();
        };
        page.envelopes.iter().filter(|e| ids.contains(&e.id)).collect()
    }

    /// Read when any target is unread, otherwise unread: one key press makes
    /// every target the same.
    pub(crate) fn toggle_read(&mut self) {
        let targets = self.target_envelopes();
        if targets.is_empty() {
            return;
        }
        let make_seen = targets.iter().any(|e| !e.flags.seen);
        let ids: Vec<MessageId> = targets.iter().map(|e| e.id).collect();
        if !self.changes_allowed_or_note() {
            return;
        }
        // The user's own choice wins over the timer for this message.
        self.auto_marked = self.current_selected_id();
        self.read_timer = None;
        self.queue.push(set_flags_for(ids, FlagChange { seen: Some(make_seen), ..FlagChange::default() }));
    }

    /// Starred when any target is not, otherwise unstarred.
    pub(crate) fn toggle_star(&mut self) {
        let targets = self.target_envelopes();
        if targets.is_empty() {
            return;
        }
        let make_flagged = targets.iter().any(|e| !e.flags.flagged);
        let ids: Vec<MessageId> = targets.iter().map(|e| e.id).collect();
        if !self.changes_allowed_or_note() {
            return;
        }
        self.queue.push(set_flags_for(ids, FlagChange { flagged: Some(make_flagged), ..FlagChange::default() }));
    }

    /// Deletes the marked messages, or the selected one, and moves the
    /// selection to the nearest message that stays, so the refreshed page
    /// keeps the user's place.
    pub(crate) fn delete_selected(&mut self) {
        let ids = self.targets();
        if ids.is_empty() {
            return;
        }
        if !self.changes_allowed_or_note() {
            return;
        }
        let neighbor = self.neighbor_outside(&ids);
        self.selection.message = neighbor;
        self.clear_marks();
        self.queue.push(Command::Delete { messages: ids });
    }

    fn current_selected_id(&self) -> Option<MessageId> {
        self.current_envelope().map(|e| e.id)
    }

    /// The nearest message that is not in `going`: the first one after the
    /// selected one, else the first before it.
    fn neighbor_outside(&self, going: &[MessageId]) -> Option<MessageId> {
        let folder = self.selection.folder?;
        let page = self.mail.page.as_ref().filter(|page| page.folder == folder)?;
        let at = self.selection.message_index;
        let stays = |e: &&Envelope| !going.contains(&e.id);
        page.envelopes
            .iter()
            .skip(at + 1)
            .find(stays)
            .or_else(|| page.envelopes.iter().take(at).rev().find(stays))
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
    set_flags_for(vec![id], change)
}

fn set_flags_for(messages: Vec<MessageId>, change: FlagChange) -> Command {
    Command::SetFlags { messages, change }
}

#[cfg(test)]
include!("changes.tests.rs");
