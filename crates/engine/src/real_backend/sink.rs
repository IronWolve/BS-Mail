//! Where the worker puts events for the UI to pick up.

use std::sync::mpsc::Sender;

use model::{AccountId, Event};

/// The sending end of the event channel, plus the account the events are
/// about.
#[derive(Debug, Clone)]
pub struct EventSink {
    sender: Sender<Event>,
    account: AccountId,
}

impl EventSink {
    pub fn new(sender: Sender<Event>, account: AccountId) -> Self {
        Self { sender, account }
    }

    pub fn account(&self) -> &AccountId {
        &self.account
    }

    /// Queues one event. A receiver that is gone means the backend was
    /// dropped, and nobody is left to tell.
    pub fn send(&self, event: Event) {
        let _ = self.sender.send(event);
    }

    pub fn send_all(&self, events: impl IntoIterator<Item = Event>) {
        for event in events {
            self.send(event);
        }
    }

    /// A failure notice for this account.
    pub fn failure(&self, operation: &str, reason: impl Into<String>) -> Event {
        Event::OpFailed {
            account: Some(self.account.clone()),
            operation: operation.to_owned(),
            reason: reason.into(),
        }
    }
}
