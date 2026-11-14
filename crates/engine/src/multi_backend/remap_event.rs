//! Puts a member's events into the global id space: every folder, message
//! and thread id gets the member's slot. Every event and every struct that
//! carries an id is listed here with no catch-all, so a new variant or a new
//! field breaks the build instead of leaking an unmapped id.

use model::{Envelope, Event, Folder, FolderId, MessageId, ThreadId};

use super::slots::{Slot, encode_folder, encode_message, encode_thread};

/// An id the namespace cannot hold: negative, or `2^40` or more.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct OutOfRange;

type Mapped<T> = Result<T, OutOfRange>;

/// `event` with every id moved into `slot`'s range.
pub(super) fn remap_event(slot: Slot, event: Event) -> Mapped<Event> {
    Ok(match event {
        Event::FolderList { account, folders } => Event::FolderList {
            account,
            folders: folders
                .into_iter()
                .map(|folder| remap_folder(slot, folder))
                .collect::<Mapped<_>>()?,
        },
        Event::EnvelopePage {
            folder,
            offset,
            total,
            envelopes,
        } => Event::EnvelopePage {
            folder: folder_id(slot, folder)?,
            offset,
            total,
            envelopes: remap_envelopes(slot, envelopes)?,
        },
        Event::OlderEnvelopes {
            folder,
            offset,
            total,
            envelopes,
        } => Event::OlderEnvelopes {
            folder: folder_id(slot, folder)?,
            offset,
            total,
            envelopes: remap_envelopes(slot, envelopes)?,
        },
        Event::BodyReady { message, body } => Event::BodyReady {
            message: message_id(slot, message)?,
            body,
        },
        Event::FlagsChanged { updates } => Event::FlagsChanged {
            updates: updates
                .into_iter()
                .map(|(message, flags)| Ok((message_id(slot, message)?, flags)))
                .collect::<Mapped<_>>()?,
        },
        Event::SearchResults {
            request,
            total,
            envelopes,
        } => Event::SearchResults {
            request,
            total,
            envelopes: remap_envelopes(slot, envelopes)?,
        },
        unchanged @ (Event::AccountList(_)
        | Event::HostInfo { .. }
        | Event::ConnectionState { .. }
        | Event::SyncProgress { .. }
        | Event::Quota { .. }
        | Event::ChangesAllowed { .. }
        | Event::MessageSent { .. }
        | Event::OpFailed { .. }) => unchanged,
    })
}

fn folder_id(slot: Slot, id: FolderId) -> Mapped<FolderId> {
    encode_folder(slot, id).ok_or(OutOfRange)
}

fn message_id(slot: Slot, id: MessageId) -> Mapped<MessageId> {
    encode_message(slot, id).ok_or(OutOfRange)
}

fn thread_id(slot: Slot, id: ThreadId) -> Mapped<ThreadId> {
    encode_thread(slot, id).ok_or(OutOfRange)
}

/// Every field is named, with no `..`, so a field added to `Folder` stops the
/// build here until it is mapped or passed on on purpose.
fn remap_folder(slot: Slot, folder: Folder) -> Mapped<Folder> {
    let Folder {
        id,
        account,
        path,
        name,
        parent,
        role,
        unread,
        total,
        expanded,
    } = folder;
    Ok(Folder {
        id: folder_id(slot, id)?,
        account,
        path,
        name,
        parent: parent.map(|parent| folder_id(slot, parent)).transpose()?,
        role,
        unread,
        total,
        expanded,
    })
}

/// Every field is named, like `remap_folder`.
fn remap_envelope(slot: Slot, envelope: Envelope) -> Mapped<Envelope> {
    let Envelope {
        id,
        folder,
        uid,
        flags,
        from,
        to,
        cc,
        subject,
        date,
        size,
        has_attachments,
        tags,
        thread,
        message_id: header_id,
    } = envelope;
    Ok(Envelope {
        id: message_id(slot, id)?,
        folder: folder_id(slot, folder)?,
        uid,
        flags,
        from,
        to,
        cc,
        subject,
        date,
        size,
        has_attachments,
        tags,
        thread: thread_id(slot, thread)?,
        message_id: header_id,
    })
}

fn remap_envelopes(slot: Slot, envelopes: Vec<Envelope>) -> Mapped<Vec<Envelope>> {
    envelopes
        .into_iter()
        .map(|envelope| remap_envelope(slot, envelope))
        .collect()
}

#[cfg(test)]
include!("remap_event.tests.rs");
