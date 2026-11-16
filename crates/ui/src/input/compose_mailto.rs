//! A mail link in a message, or on the command line, opens the composer: the
//! address (and the subject and text the link asks for) filled in, for the
//! user to read and send. Nothing is sent by a link. The program registers
//! itself as no system's mail handler (that would write outside its own
//! folder); a `mailto:` argument at start is how another program hands it a
//! link.

use super::compose_start::READ_ONLY_NOTE;
use crate::app_view::AppView;
use crate::composer::reply::Draft;
use crate::composer::{ComposeForm, MailtoDraft, parse_mailto};

impl AppView {
    /// Opens the composer for the link or address `text`; a link that cannot
    /// be read says why in the footer.
    pub(crate) fn open_mail_link(&mut self, text: &str) {
        match parse_mailto(text) {
            Ok(link) => {
                self.start_mailto(link);
            }
            Err(error) => self.mail.set_note(Some(error.words())),
        }
    }

    /// Opens the composer for `link`, from the account on screen when it may
    /// send, else the first that may. Returns whether it opened.
    pub(crate) fn start_mailto(&mut self, link: MailtoDraft) -> bool {
        let senders = self.writable_senders();
        if senders.is_empty() {
            if !self.mail.accounts.is_empty() {
                self.mail.set_note(Some(READ_ONLY_NOTE.to_string()));
            }
            return false;
        }
        let from = self.sender_of(&senders, None);
        let chosen = &senders[from];
        let draft = Draft::for_link(link.to, link.cc, link.subject, link.body);
        let form = ComposeForm::from_draft(chosen.account.clone(), chosen.from.clone(), &draft, None)
            .with_senders(senders.clone(), from, true)
            .with_bcc(&link.bcc);
        self.composer = Some(form);
        true
    }

    /// The program was started with a `mailto:` link: the composer opens for
    /// it as soon as the first account's mail is here to write from.
    pub fn start_with_mailto(&mut self, text: &str) {
        match parse_mailto(text) {
            Ok(link) => self.pending_mailto = Some(link),
            Err(error) => self.mail.set_note(Some(error.words())),
        }
    }

    /// Opens the composer for the link the program was started with, once an
    /// account is open; a screen with no account to write from drops it.
    pub(crate) fn try_pending_mailto(&mut self) {
        if self.pending_mailto.is_none() || self.selection.folder.is_none() {
            return;
        }
        if let Some(link) = self.pending_mailto.take() {
            self.start_mailto(link);
        }
    }
}

#[cfg(test)]
include!("compose_mailto.tests.rs");
