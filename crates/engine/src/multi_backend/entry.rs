//! A member in service: the backend with its slot, what the composite has
//! learned about it, and events waiting to be fed through.

use model::{AccountId, Event, FolderRole, MailBackend};

use super::cache::MemberCache;
use super::member::Member;
use super::slots::Slot;

pub(super) struct Entry {
    pub slot: Slot,
    pub id: AccountId,
    pub name: String,
    pub address: String,
    pub backend: Box<dyn MailBackend>,
    pub cache: MemberCache,
    /// Events the member produced before it was added; they go through the
    /// same path as polled ones, at the next poll.
    pub backlog: Vec<Event>,
    /// An out-of-range id was already reported for this member.
    pub range_warned: bool,
    /// The Inbox unread count the member's latest folder list said, once it
    /// has said one.
    pub inbox_unread: Option<u32>,
}

impl Entry {
    pub fn new(slot: Slot, member: Member, early: Vec<Event>) -> Self {
        let Member {
            id,
            name,
            address,
            backend,
        } = member;
        Self {
            slot,
            id,
            name,
            address,
            backend,
            cache: MemberCache::default(),
            backlog: early,
            range_warned: false,
            inbox_unread: None,
        }
    }

    /// Takes in the Inbox count a folder list says. The event that tells the
    /// screen when it differs from the one last taken in; nothing for any
    /// other event, a list with no Inbox, or the same count again.
    pub fn note_inbox_unread(&mut self, event: &Event) -> Option<Event> {
        let Event::FolderList { folders, .. } = event else {
            return None;
        };
        let unread = folders.iter().find(|folder| folder.role == FolderRole::Inbox)?.unread;
        if self.inbox_unread == Some(unread) {
            return None;
        }
        self.inbox_unread = Some(unread);
        Some(Event::AccountUnread { account: self.id.clone(), unread })
    }
}
