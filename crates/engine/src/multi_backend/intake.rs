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

/// The events of `events` that go to the screen, in order.
pub(super) fn admit_all(entry: &mut Entry, events: Vec<Event>, gate: &mut Gate<'_>) -> Vec<Event> {
    events
        .into_iter()
        .filter_map(|event| admit(entry, event, gate))
        .collect()
}

fn admit(entry: &mut Entry, event: Event, gate: &mut Gate<'_>) -> Option<Event> {
    let class = classify(&event);
    if class == Class::Swallowed {
        return None;
    }
    let event = match remap_event(entry.slot, event) {
        Ok(event) => event,
        Err(OutOfRange) => return out_of_range_notice(entry),
    };
    entry.cache.store(&event);
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
