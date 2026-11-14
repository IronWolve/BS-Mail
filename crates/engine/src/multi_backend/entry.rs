//! A member in service: the backend with its slot, what the composite has
//! learned about it, and events waiting to be fed through.

use model::{AccountId, Event, MailBackend};

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
        }
    }
}
