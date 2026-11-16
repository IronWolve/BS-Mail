//! The notice of new mail, inside the program only: no bell, no desktop
//! notification and no sound. When an account's Inbox unread count goes up
//! after the first count the program knew of, the footer says
//! `New mail: N in <address>` for about eight seconds, or until the next key,
//! whichever comes first. `[notify] new_mail = false` switches it off.
//!
//! A count that goes up because the user marked mail unread or moved mail into
//! the Inbox is not new mail: for a few seconds after the program sent such a
//! change, a rise in the account on screen is taken for that change.

use model::{AccountId, Command, Event};

use crate::app_view::AppView;

/// How long the notice stays when no key is pressed.
pub const NOTICE_MS: u64 = 8_000;

/// How long after the user's own change a rise in the account on screen is
/// put down to it.
const OWN_CHANGE_MS: u64 = 5_000;

/// The notice on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMail {
    account: AccountId,
    /// New messages in all, while the notice has stayed up for this account.
    count: u32,
    text: String,
    until_ms: u64,
}

impl AppView {
    /// The words of the notice of new mail, while there is one.
    pub fn new_mail_notice(&self) -> Option<&str> {
        self.new_mail.as_ref().map(|notice| notice.text.as_str())
    }

    /// For an `AccountUnread` event not yet applied: the account and by how
    /// many its Inbox unread count has gone up since the count the program
    /// held. Nothing for the first count of an account, or a fall.
    pub(crate) fn unread_rise(&self, event: &Event) -> Option<(AccountId, u32)> {
        let Event::AccountUnread { account, unread } = event else {
            return None;
        };
        let before = self.mail.account(account)?.unread?;
        unread.checked_sub(before).filter(|by| *by > 0).map(|by| (account.clone(), by))
    }

    /// An account's Inbox has `by` more unread messages: says so in the
    /// footer, unless the notice is off or the user's own change explains it.
    pub(crate) fn note_new_mail(&mut self, account: &AccountId, by: u32) {
        if !self.config.notify.new_mail || self.own_change_explains(account) {
            return;
        }
        let Some(listed) = self.mail.account(account) else {
            return;
        };
        let name = if listed.address.is_empty() { &listed.name } else { &listed.address };
        let carried = self.new_mail.as_ref().filter(|notice| &notice.account == account).map_or(0, |notice| notice.count);
        let count = carried.saturating_add(by);
        self.new_mail = Some(NewMail {
            account: account.clone(),
            count,
            text: format!("New mail: {count} in {name}"),
            until_ms: self.last_tick_ms.saturating_add(NOTICE_MS),
        });
    }

    fn own_change_explains(&self, account: &AccountId) -> bool {
        let shown = self.selection.account.as_ref() == Some(account);
        let recent = self.own_change_ms.is_some_and(|at| self.last_tick_ms.saturating_sub(at) <= OWN_CHANGE_MS);
        shown && recent
    }

    /// Notes that the commands about to go to the backend change mail.
    pub(crate) fn note_own_changes(&mut self, commands: &[Command]) {
        let changes = commands
            .iter()
            .any(|command| matches!(command, Command::SetFlags { .. } | Command::Move { .. } | Command::Delete { .. }));
        if changes {
            self.own_change_ms = Some(self.last_tick_ms);
        }
    }

    /// The notice goes once its time is up.
    pub(crate) fn expire_new_mail(&mut self, now_ms: u64) {
        if self.new_mail.as_ref().is_some_and(|notice| now_ms >= notice.until_ms) {
            self.new_mail = None;
        }
    }
}

#[cfg(test)]
include!("new_mail.tests.rs");
