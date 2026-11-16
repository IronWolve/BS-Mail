//! The From line of the form: the accounts it can be set to, changing it, and
//! what changing it does to the signature in the body.

use super::form::{ComposeForm, Field};
use super::sender::{Sender, neighbour};
use super::signature;

impl ComposeForm {
    /// Sets the accounts the From line can take and which of them the message
    /// is from. With `sign`, the body gets that account's signature, as
    /// `signature::insert` places it, and takes the next one's when the From
    /// line changes. The form counts the body as unwritten still: only what
    /// the user adds makes it something to ask about before closing.
    #[must_use]
    pub fn with_senders(mut self, senders: Vec<Sender>, current: usize, sign: bool) -> Self {
        let Some(chosen) = senders.get(current).cloned() else {
            return self;
        };
        self.senders = senders;
        self.sender = current;
        self.account = chosen.account;
        self.from = chosen.from;
        self.from_line.set_text(&self.from);
        self.signs = sign;
        if sign {
            self.opening_body = signature::insert(&self.base_body, &chosen.signature);
            self.body.set_text(&self.opening_body);
            self.body.move_text_start(false);
        }
        self.started_with = self.fingerprint();
        self
    }

    /// Whether the From line has more than one account to choose from.
    pub fn can_choose_sender(&self) -> bool {
        self.senders.len() > 1
    }

    /// The From line set to the account `step` away in the list, round the
    /// ends. False when there is no other account.
    pub fn step_sender(&mut self, forward: bool) -> bool {
        self.choose_sender(neighbour(self.senders.len(), self.sender, forward))
    }

    /// The From line set to the `index`th account. The signature of the one
    /// left is swapped for the new one's. False when nothing changed.
    pub fn choose_sender(&mut self, index: usize) -> bool {
        let (Some(old), Some(new)) = (self.senders.get(self.sender).cloned(), self.senders.get(index).cloned()) else {
            return false;
        };
        if index == self.sender {
            return false;
        }
        self.sender = index;
        self.account = new.account;
        self.from_line.set_text(&new.from);
        self.from = new.from;
        self.swap_signature(&old.signature, &new.signature);
        true
    }

    /// An untouched body is made again for the new account when it was made
    /// with a signature; any other body has the old signature replaced where
    /// it stands, when it is still there whole.
    fn swap_signature(&mut self, old: &str, new: &str) {
        let text = self.body.text();
        let untouched = text == self.opening_body;
        let next = if untouched && self.signs {
            Some(signature::insert(&self.base_body, new))
        } else {
            signature::swap(&text, old, new)
        };
        let Some(next) = next else {
            return;
        };
        let at = self.body.cursor();
        self.body.set_text(&next);
        if untouched {
            self.body.move_text_start(false);
            self.opening_body = next;
            self.started_with = self.fingerprint();
        } else {
            self.body.place_near(at);
        }
    }

    /// The field after or before the focused one, leaving out the From line
    /// when there is no other account to choose and the Attach row while there
    /// is nothing on it.
    pub fn step_focus(&mut self, forward: bool) {
        let mut field = self.focus;
        loop {
            field = if forward { field.next() } else { field.previous() };
            let usable = match field {
                Field::From => self.can_choose_sender(),
                Field::Attach => self.can_focus_attach(),
                _ => true,
            };
            if usable {
                break;
            }
        }
        self.focus = field;
    }

    /// What the title row says the message is.
    pub fn title(&self) -> &'static str {
        if self.draft.is_some() {
            "Draft"
        } else if self.in_reply_to.is_some() {
            "Reply"
        } else {
            "New message"
        }
    }
}

#[cfg(test)]
include!("form_sender.tests.rs");
