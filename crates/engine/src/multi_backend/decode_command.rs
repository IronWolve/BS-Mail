//! Turns a command from the screen, which carries global ids, into what each
//! member must receive, which carries its own ids. Every command is listed
//! with no catch-all, so a new command must be routed here before this
//! builds.

use std::collections::BTreeMap;

use model::{AccountId, Command, FolderId, MessageId, OutgoingMessage};

use super::slots::{Slot, decode_folder, decode_message};

/// Why a move across accounts is refused.
pub(super) const CROSS_ACCOUNT_MOVE: &str = "mail cannot be moved between accounts";

/// Why a new folder whose parent is in another account is refused.
const CROSS_ACCOUNT_FOLDER: &str = "a folder cannot be made inside another account's folder";

/// Why a command naming an id that this numbering never produced is refused.
const NOT_AN_ID: &str = "that is not an id of any account's mail";

/// Why a command naming an account that is not a member is refused.
pub(super) fn unknown_account_reason(account: &AccountId) -> String {
    format!("the account {} is no longer there", account.0)
}

/// Finds the slot of the member that serves an account.
pub(super) type SlotOf<'a> = &'a dyn Fn(&AccountId) -> Option<Slot>;

/// What to do with one command.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Plan {
    /// Commands for members, with their own ids, in slot order.
    Forward(Vec<(Slot, Command)>),
    /// The same command for every member.
    Everyone(Command),
    /// The user switched to this account.
    Select(AccountId),
    /// Not carried out; answer with one failure that names no account.
    Refused {
        operation: &'static str,
        reason: String,
    },
}

impl Plan {
    fn to(slot: Slot, command: Command) -> Self {
        Self::Forward(vec![(slot, command)])
    }

    fn refused(operation: &'static str, reason: impl Into<String>) -> Self {
        Self::Refused {
            operation,
            reason: reason.into(),
        }
    }

    fn not_an_id() -> Self {
        Self::refused("command", NOT_AN_ID)
    }

    fn unknown_account(operation: &'static str, account: &AccountId) -> Self {
        Self::refused(operation, unknown_account_reason(account))
    }
}

pub(super) fn plan(command: Command, slot_of: SlotOf<'_>) -> Plan {
    match command {
        Command::OpenFolder { folder } => match decode_folder(folder) {
            Some((slot, folder)) => Plan::to(slot, Command::OpenFolder { folder }),
            None => Plan::not_an_id(),
        },
        Command::LoadMore { folder, have } => match decode_folder(folder) {
            Some((slot, folder)) => Plan::to(slot, Command::LoadMore { folder, have }),
            None => Plan::not_an_id(),
        },
        Command::FetchBody { message } => match decode_message(message) {
            Some((slot, message)) => Plan::to(slot, Command::FetchBody { message }),
            None => Plan::not_an_id(),
        },
        Command::FetchRaw { message, part } => match decode_message(message) {
            Some((slot, message)) => Plan::to(slot, Command::FetchRaw { message, part }),
            None => Plan::not_an_id(),
        },
        Command::PrepareHtml { message } => match decode_message(message) {
            Some((slot, message)) => Plan::to(slot, Command::PrepareHtml { message }),
            None => Plan::not_an_id(),
        },
        Command::FetchAttachment { message, attachment } => match decode_message(message) {
            Some((slot, message)) => Plan::to(slot, Command::FetchAttachment { message, attachment }),
            None => Plan::not_an_id(),
        },
        Command::SetFlags { messages, change } => {
            split_by_slot(messages, |messages| Command::SetFlags { messages, change })
        }
        Command::Delete { messages } => {
            split_by_slot(messages, |messages| Command::Delete { messages })
        }
        Command::SetTag { messages, tag, on } => {
            split_by_slot(messages, |messages| Command::SetTag { messages, tag: tag.clone(), on })
        }
        Command::Move { messages, to } => plan_move(messages, to),
        Command::CreateFolder { account, parent, name } => plan_create_folder(account, parent, name, slot_of),
        Command::RenameFolder { folder, name } => {
            on_folder(folder, |folder| Command::RenameFolder { folder, name })
        }
        Command::DeleteFolder { folder } => on_folder(folder, |folder| Command::DeleteFolder { folder }),
        Command::MarkAllRead { folder } => on_folder(folder, |folder| Command::MarkAllRead { folder }),
        Command::EmptyFolder { folder } => on_folder(folder, |folder| Command::EmptyFolder { folder }),
        Command::Send { message } => plan_send(message, slot_of),
        Command::Search {
            request,
            account,
            query,
        } => for_account(account, slot_of, "command", |account| Command::Search {
            request,
            account,
            query,
        }),
        Command::SyncNow { account } => {
            for_account(account, slot_of, "command", |account| Command::SyncNow { account })
        }
        Command::SelectAccount { account } => Plan::Select(account),
        unlock @ Command::Unlock { .. } => Plan::Everyone(unlock),
    }
}

/// Groups messages by the member that owns them, each group in the order it
/// came, groups in slot order. One id that cannot be decoded refuses the
/// whole command.
fn split_by_slot(messages: Vec<MessageId>, make: impl Fn(Vec<MessageId>) -> Command) -> Plan {
    let mut groups: BTreeMap<Slot, Vec<MessageId>> = BTreeMap::new();
    for message in messages {
        let Some((slot, local)) = decode_message(message) else {
            return Plan::not_an_id();
        };
        groups.entry(slot).or_default().push(local);
    }
    Plan::Forward(groups.into_iter().map(|(slot, ids)| (slot, make(ids))).collect())
}

/// A move stays inside one account: the target folder and every message must
/// belong to the same member, or nothing is sent.
fn plan_move(messages: Vec<MessageId>, to: FolderId) -> Plan {
    let Some((slot, target)) = decode_folder(to) else {
        return Plan::not_an_id();
    };
    let mut local = Vec::with_capacity(messages.len());
    for message in messages {
        match decode_message(message) {
            Some((owner, id)) if owner == slot => local.push(id),
            Some(_) => return Plan::refused("move", CROSS_ACCOUNT_MOVE),
            None => return Plan::not_an_id(),
        }
    }
    Plan::to(slot, Command::Move { messages: local, to: target })
}

/// A command about one folder goes to the member that owns it, with the
/// folder's own id.
fn on_folder(folder: FolderId, make: impl FnOnce(FolderId) -> Command) -> Plan {
    match decode_folder(folder) {
        Some((slot, local)) => Plan::to(slot, make(local)),
        None => Plan::not_an_id(),
    }
}

/// A new folder goes to the member of the account it is made in; its parent,
/// when it has one, must belong to that member too, or nothing is sent.
fn plan_create_folder(account: AccountId, parent: Option<FolderId>, name: String, slot_of: SlotOf<'_>) -> Plan {
    let Some(slot) = slot_of(&account) else {
        return Plan::unknown_account("new folder", &account);
    };
    let parent = match parent.map(decode_folder) {
        None => None,
        Some(Some((owner, local))) if owner == slot => Some(local),
        Some(Some(_)) => return Plan::refused("new folder", CROSS_ACCOUNT_FOLDER),
        Some(None) => return Plan::not_an_id(),
    };
    Plan::to(slot, Command::CreateFolder { account, parent, name })
}

/// A send goes to the member of the message's account. A refusal is named
/// `send` because that is the name the composer listens for. The message it
/// answers is cleared when it belongs to another member, since that member
/// cannot look it up.
fn plan_send(message: OutgoingMessage, slot_of: SlotOf<'_>) -> Plan {
    let Some(slot) = slot_of(&message.account) else {
        return Plan::unknown_account("send", &message.account);
    };
    let OutgoingMessage {
        account,
        to,
        cc,
        bcc,
        subject,
        body,
        in_reply_to,
        attachments,
    } = message;
    let in_reply_to = in_reply_to
        .and_then(decode_message)
        .filter(|(owner, _)| *owner == slot)
        .map(|(_, local)| local);
    let message = OutgoingMessage {
        account,
        to,
        cc,
        bcc,
        subject,
        body,
        in_reply_to,
        attachments,
    };
    Plan::to(slot, Command::Send { message })
}

fn for_account(
    account: AccountId,
    slot_of: SlotOf<'_>,
    operation: &'static str,
    make: impl FnOnce(AccountId) -> Command,
) -> Plan {
    match slot_of(&account) {
        Some(slot) => Plan::to(slot, make(account)),
        None => Plan::unknown_account(operation, &account),
    }
}

#[cfg(test)]
include!("decode_command.tests.rs");
