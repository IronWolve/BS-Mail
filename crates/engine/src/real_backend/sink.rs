//! Where the worker puts events for the UI to pick up.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use model::{AccountId, ConnectionState, Event};

/// The sending end of the event channel, plus the account the events are
/// about. It also keeps track of whether the connection is up, from the
/// connection events that pass through, so the owner can turn away a change
/// at once while the connection is down.
#[derive(Debug, Clone)]
pub struct EventSink {
    sender: Sender<Event>,
    account: AccountId,
    online: Arc<AtomicBool>,
}

impl EventSink {
    pub fn new(sender: Sender<Event>, account: AccountId) -> Self {
        Self {
            sender,
            account,
            online: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn account(&self) -> &AccountId {
        &self.account
    }

    /// A handle on the connection flag: true while the account is online.
    pub fn online_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.online)
    }

    /// Queues one event. A receiver that is gone means the backend was
    /// dropped, and nobody is left to tell.
    pub fn send(&self, event: Event) {
        if let Event::ConnectionState { state, .. } = &event {
            self.online
                .store(*state == ConnectionState::Online, Ordering::Release);
        }
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
