//! Changing mail from the screen: read and unread, the star, delete, and
//! marking a message read once it has been open long enough.
//!
//! Each of these only asks the backend. The screen changes when the backend
//! says it happened, so a refusal or a failure never leaves it showing
//! something the server does not have.

use std::collections::HashSet;

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
    pub(crate) fn changes_allowed_or_note(&mut self) -> bool {
        if self.mail.changes_refused {
            self.mail.set_note(Some(READ_ONLY_NOTE.to_string()));
            return false;
        }
        true
    }

    /// The envelopes of the messages a change acts on, found among every
    /// loaded message (a thread's folded messages have no row of their own).
    pub(crate) fn target_envelopes(&self) -> Vec<&Envelope> {
        let ids: HashSet<MessageId> = self.targets().into_iter().collect();
        let Some(page) = self.shown_page() else {
            return Vec::new();
        };
        page.loaded.iter().filter(|e| ids.contains(&e.id)).collect()
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

    /// A click on the unread cell of a row: the messages the row stands for
    /// (that message alone, or every message of a thread's row) are made read
    /// if any is unread, unread if all are read.
    pub(crate) fn toggle_read_of(&mut self, ids: &[MessageId]) {
        let found: Vec<&Envelope> = ids.iter().filter_map(|id| self.envelope_of(*id)).collect();
        if found.is_empty() {
            return;
        }
        let make_seen = found.iter().any(|envelope| !envelope.flags.seen);
        if !self.changes_allowed_or_note() {
            return;
        }
        // The user's own choice wins over the timer for the message open.
        if let Some(open) = self.selection.message.filter(|open| ids.contains(open)) {
            self.auto_marked = Some(open);
            self.read_timer = None;
        }
        self.queue.push(set_flags_for(ids.to_vec(), FlagChange { seen: Some(make_seen), ..FlagChange::default() }));
    }

    /// A click on the star cell of a row: the messages the row stands for are
    /// starred if any is not, unstarred if all are.
    pub(crate) fn toggle_star_of(&mut self, ids: &[MessageId]) {
        let found: Vec<&Envelope> = ids.iter().filter_map(|id| self.envelope_of(*id)).collect();
        if found.is_empty() {
            return;
        }
        let make_flagged = found.iter().any(|envelope| !envelope.flags.flagged);
        if !self.changes_allowed_or_note() {
            return;
        }
        self.queue.push(set_flags_for(ids.to_vec(), FlagChange { flagged: Some(make_flagged), ..FlagChange::default() }));
    }

    fn envelope_of(&self, id: MessageId) -> Option<&Envelope> {
        self.shown_page()?.loaded_envelope(id)
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

    /// Asks before deleting the marked messages, or the selected one. A yes
    /// (see `delete_prompt`) moves them to the Trash, or removes them for good
    /// in the Trash, and moves the selection to the nearest message that
    /// stays, so the refreshed page keeps the user's place. Nothing is sent
    /// before the answer.
    pub(crate) fn delete_selected(&mut self) {
        let ids = self.targets();
        if ids.is_empty() {
            return;
        }
        if !self.changes_allowed_or_note() {
            return;
        }
        let for_good = self.count_deleted_for_good(&ids);
        self.ask_before_deleting(ids, for_good);
    }

    fn current_selected_id(&self) -> Option<MessageId> {
        self.current_envelope().map(|e| e.id)
    }

    /// The row and id of the nearest message that is not in `going`: the first
    /// one after the selected one, else the first before it.
    fn neighbor_outside(&self, going: &[MessageId]) -> Option<(usize, MessageId)> {
        let page = self.shown_page()?;
        let at = self.selection.message_index;
        let stays = |(_, e): &(usize, &Envelope)| !going.contains(&e.id);
        let rows = || page.envelopes.iter().enumerate();
        rows()
            .skip(at + 1)
            .find(stays)
            .or_else(|| rows().take(at).rev().find(stays))
            .map(|(row, envelope)| (row, envelope.id))
    }

    /// Moves the selection to the nearest message that stays when `going` are
    /// moved or deleted, when the selected message is one of them. The row and the id move together, so the highlight,
    /// the headers and the body all name the neighbor from here on, whether
    /// the change is carried out (its page then keeps the place) or refused
    /// (nothing arrives, and the neighbor is still the one shown). With no
    /// message left to go to, the selection stays where it is.
    pub(crate) fn select_neighbor_outside(&mut self, going: &[MessageId]) {
        // The selection only moves off a message that leaves: marks made with
        // Space leave the cursor on the first message after the run, and a
        // change to the marked ones must not push it on from there.
        if !self.selection.message.is_some_and(|selected| going.contains(&selected)) {
            return;
        }
        let Some((row, id)) = self.neighbor_outside(going) else {
            return;
        };
        if self.selection.message != Some(id) {
            self.scroll.reader = 0;
        }
        self.selection.message = Some(id);
        self.selection.message_index = row;
    }

    /// Marks the selected message read once its body has been showing for
    /// `reader.mark_read_after_ms`. `now_ms` is any steady clock in
    /// milliseconds; the caller calls this a few times a second.
    pub fn tick(&mut self, now_ms: u64) {
        self.last_tick_ms = now_ms;
        self.set_clock(now_ms);
        self.expire_new_mail(now_ms);
        if self.options.is_some() || self.composer.is_some() || self.setup.is_some() || self.overlays.is_open() {
            // Mail that is hidden behind the Options screen, the composer, the
            // account form or the help is not marked read, and its time starts
            // over when it shows again.
            self.read_timer = None;
            return;
        }
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
