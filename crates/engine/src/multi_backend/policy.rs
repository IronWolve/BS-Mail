//! What the composite does with each kind of event a member sends. The match
//! lists every event, so a new kind must be placed here before this builds.

use model::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Class {
    /// The composite builds the account list and the Inbox unread counts
    /// itself (from the folder lists the members send); a member's own is
    /// dropped.
    Swallowed,
    /// Forwarded once, from whichever member says it first.
    Host,
    /// Kept, and always forwarded, so every account's dot stays live.
    Connection,
    /// About the account on screen: kept always, forwarded only from the
    /// active member.
    Status,
    /// Answers a question the screen asks about the account it shows: dropped
    /// when the member is not the active one.
    Answer,
    /// Always forwarded: a send that finishes after the user switched must
    /// still answer the composer, and failures must show.
    Notice,
}

pub(super) fn classify(event: &Event) -> Class {
    match event {
        Event::AccountList(_) | Event::AccountUnread { .. } => Class::Swallowed,
        Event::HostInfo { .. } => Class::Host,
        Event::ConnectionState { .. } => Class::Connection,
        Event::FolderList { .. }
        | Event::ChangesAllowed { .. }
        | Event::Quota { .. }
        | Event::SyncProgress { .. } => Class::Status,
        Event::EnvelopePage { .. }
        | Event::OlderEnvelopes { .. }
        | Event::BodyReady { .. }
        | Event::FlagsChanged { .. }
        | Event::SearchResults { .. } => Class::Answer,
        Event::MessageSent { .. } | Event::OpFailed { .. } => Class::Notice,
    }
}

impl Class {
    /// Whether an event of this class goes to the screen. `active` is whether
    /// the member that sent it is the active account. `Swallowed` and `Host`
    /// are decided by the caller.
    pub fn forwarded(self, active: bool) -> bool {
        match self {
            Self::Swallowed | Self::Host => false,
            Self::Connection | Self::Notice => true,
            Self::Status | Self::Answer => active,
        }
    }
}

#[cfg(test)]
include!("policy.tests.rs");
