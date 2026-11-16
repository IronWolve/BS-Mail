//! What happens to each event a member produces, whether it came from a poll,
//! from a command's answer, or was already waiting when the member was
//! added: the ids are remapped, the screen's state about the account is
//! kept, and the event is forwarded or dropped by the rules in `policy`.

use model::{AccountId, Event};

use super::entry::Entry;
use super::policy::{Class, classify};
use super::remap_event::{OutOfRange, remap_event};

/// What the composite knows that decides whether an event is forwarded.
pub(super) struct Gate<'a> {
    /// The account the screen is showing.
    pub active: Option<&'a AccountId>,
    /// The host description was already forwarded.
    pub host_sent: &'a mut bool,
}

/// The events of `events` that go to the screen, in order. A folder list that
/// changes the member's Inbox unread count is followed by the event that says
/// so, whether or not the list itself goes on: the screen shows every
/// account's count.
pub(super) fn admit_all(entry: &mut Entry, events: Vec<Event>, gate: &mut Gate<'_>) -> Vec<Event> {
    let mut out = Vec::new();
    for event in events {
        let class = classify(&event);
        if class == Class::Swallowed {
            continue;
        }
        let event = match remap_event(entry.slot, event) {
            Ok(event) => event,
            Err(OutOfRange) => {
                out.extend(out_of_range_notice(entry));
                continue;
            }
        };
        entry.cache.store(&event);
        let unread = entry.note_inbox_unread(&event);
        out.extend(forward(entry, event, class, gate));
        out.extend(unread);
    }
    keep_the_last_count(out)
}

/// Of the counts one batch tells about an account only the last stands: the
/// account list built after the batch already carries it, and a count that
/// went up and then up again must not look to the screen like a fall and then
/// a rise.
fn keep_the_last_count(events: Vec<Event>) -> Vec<Event> {
    let last = events.iter().rposition(|event| matches!(event, Event::AccountUnread { .. }));
    events
        .into_iter()
        .enumerate()
        .filter(|(at, event)| !matches!(event, Event::AccountUnread { .. }) || Some(*at) == last)
        .map(|(_, event)| event)
        .collect()
}

fn forward(entry: &Entry, event: Event, class: Class, gate: &mut Gate<'_>) -> Option<Event> {
    match class {
        Class::Host => (!std::mem::replace(gate.host_sent, true)).then_some(event),
        other => other
            .forwarded(gate.active == Some(&entry.id))
            .then_some(event),
    }
}

/// The one notice a member earns for sending an id the numbering cannot hold.
/// The event itself is dropped; later ones are dropped without a notice.
fn out_of_range_notice(entry: &mut Entry) -> Option<Event> {
    if std::mem::replace(&mut entry.range_warned, true) {
        return None;
    }
    Some(Event::OpFailed {
        account: Some(entry.id.clone()),
        operation: "account".to_owned(),
        reason: "the account sent an id that cannot be told apart from another account's mail; \
                 that mail is not shown"
            .to_owned(),
    })
}

#[cfg(test)]
include!("intake.tests.rs");
