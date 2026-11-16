//! Starting the composer from the screen: which accounts the message can be
//! from, which one it starts with, and what it starts with.
//!
//! A new message is from the account on screen, a reply, a forward, a draft or
//! a copy to edit is from the account the message belongs to (the folder it is
//! filed in says which), and when that account may not send it starts with
//! the first account that may. The From line then changes among the accounts
//! that may send.

use model::{AccountId, FolderRole, MessageId};

use super::compose::ComposeKind;
use crate::app_view::AppView;
use crate::composer::reply::{Draft, Original, body_text, edit_as_new, forward, reply};
use crate::composer::{ComposeForm, Sender};
use crate::format::{CivilTime, format_pattern};

/// What the footer says when no account allows sending.
const READ_ONLY_NOTE: &str = "this account is read-only: set read_only = false in its file to send mail";

/// What the footer says when the message to answer has not arrived yet: its
/// Reply-To and its text come with its body, and a reply made without them
/// could go to the wrong address.
pub(super) const BODY_NOT_HERE_NOTE: &str =
    "this message has not finished loading, so it cannot be answered, forwarded or edited yet; try again in a moment";

/// Why no draft could be made from the selected message.
enum NoDraft {
    /// No message is selected, or it is not one this kind of message can be
    /// made from.
    NothingSelected,
    /// Its body, which holds the Reply-To and the text, has not arrived.
    BodyNotHere,
}

/// What a message starts with, and whose it is.
struct Started {
    draft: Draft,
    reply_to: Option<MessageId>,
    /// The draft on the server this message goes on from.
    continues: Option<MessageId>,
    /// Where in the list of senders the message starts.
    from: usize,
}

impl AppView {
    /// Opens the composer for a new message, or for a reply, a forward, a
    /// draft to go on with or a copy of the selected message to edit.
    pub(crate) fn start_compose(&mut self, kind: ComposeKind) {
        let senders = self.writable_senders();
        if senders.is_empty() {
            if !self.mail.accounts.is_empty() {
                self.mail.set_note(Some(READ_ONLY_NOTE.to_string()));
            }
            return;
        }
        let started = match kind {
            ComposeKind::New => Started { draft: Draft::default(), reply_to: None, continues: None, from: self.sender_of(&senders, None) },
            other => match self.started_from_selected(other, &senders) {
                Ok(started) => started,
                Err(NoDraft::NothingSelected) => return,
                Err(NoDraft::BodyNotHere) => {
                    self.mail.set_note(Some(BODY_NOT_HERE_NOTE.to_string()));
                    return;
                }
            },
        };
        let chosen = &senders[started.from];
        let signs = !matches!(kind, ComposeKind::Draft | ComposeKind::EditAsNew);
        let mut form = ComposeForm::from_draft(chosen.account.clone(), chosen.from.clone(), &started.draft, started.reply_to)
            .with_senders(senders.clone(), started.from, signs);
        form.draft = started.continues;
        self.composer = Some(form);
    }

    /// Whether the selected message is filed in a Drafts folder, which Enter
    /// goes on writing instead of opening it to read.
    pub(crate) fn selected_is_draft(&self) -> bool {
        let Some(page) = self.shown_page() else {
            return false;
        };
        let Some(envelope) = page.envelopes.get(self.selection.message_index) else {
            return false;
        };
        self.mail.folder(envelope.folder).is_some_and(|filed| filed.role == FolderRole::Drafts)
    }

    /// Whether mail may be written from the account: it did not say it is
    /// read-only, and it is not the account on screen with changes refused.
    fn account_may_send(&self, id: &AccountId) -> bool {
        let shown = self.selection.account.as_ref() == Some(id);
        self.mail.may_send_from(id) && !(shown && self.mail.changes_refused)
    }

    /// The accounts a message can be from, in the order of the account list.
    fn writable_senders(&self) -> Vec<Sender> {
        self.mail
            .accounts
            .iter()
            .filter(|account| self.account_may_send(&account.id))
            .map(|account| Sender::new(account.id.clone(), &account.name, &account.address, self.mail.signature_of(&account.id)))
            .collect()
    }

    /// Where in `senders` a message about `account` starts: that account, else
    /// the one on screen, else the first.
    fn sender_of(&self, senders: &[Sender], account: Option<&AccountId>) -> usize {
        let position = |id: &AccountId| senders.iter().position(|sender| &sender.account == id);
        account.and_then(position).or_else(|| self.selection.account.as_ref().and_then(position)).unwrap_or(0)
    }

    /// What a reply, a forward, a draft or a copy of the selected message
    /// starts with. Only once its body is here: without it the draft would
    /// have no Reply-To and no quoted text.
    fn started_from_selected(&self, kind: ComposeKind, senders: &[Sender]) -> Result<Started, NoDraft> {
        let folder = self.selection.folder.ok_or(NoDraft::NothingSelected)?;
        let page = self.mail.page.as_ref().filter(|page| page.folder == folder).ok_or(NoDraft::NothingSelected)?;
        let envelope = page.envelopes.get(self.selection.message_index).ok_or(NoDraft::NothingSelected)?;
        let own_folder = self.mail.folder(envelope.folder);
        if kind == ComposeKind::Draft && own_folder.is_none_or(|filed| filed.role != FolderRole::Drafts) {
            return Err(NoDraft::NothingSelected);
        }
        let body = self
            .mail
            .body
            .as_ref()
            .filter(|(id, _)| *id == envelope.id)
            .map(|(_, body)| body)
            .ok_or(NoDraft::BodyNotHere)?;
        let time = CivilTime::from_unix(envelope.date.unix, envelope.date.offset_minutes);
        let original = Original {
            envelope,
            text: body_text(body),
            date: format_pattern(&self.config.reader.date_format, &time),
            reply_to: body.reply_to.clone(),
        };
        let from = self.sender_of(senders, own_folder.map(|filed| &filed.account));
        let own = senders[from].address.as_str();
        let (draft, reply_to, continues) = match kind {
            ComposeKind::Forward => (forward(&original), None, None),
            ComposeKind::ReplyAll => (reply(&original, own, true), Some(envelope.id), None),
            ComposeKind::EditAsNew => (edit_as_new(&original), None, None),
            ComposeKind::Draft => (edit_as_new(&original), None, Some(envelope.id)),
            ComposeKind::Reply => (reply(&original, own, false), Some(envelope.id), None),
            ComposeKind::New => (Draft::default(), None, None),
        };
        Ok(Started { draft, reply_to, continues, from })
    }
}

#[cfg(test)]
include!("compose_start.tests.rs");
