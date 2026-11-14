//! A backend with no mail: what the screen talks to while the account form
//! is open and there is nothing yet to ask for.

use model::{Command, Event, MailBackend};

#[derive(Debug, Default)]
pub struct NullBackend;

impl MailBackend for NullBackend {
    fn handle(&mut self, _command: Command) -> Vec<Event> {
        Vec::new()
    }

    fn poll_events(&mut self) -> Vec<Event> {
        Vec::new()
    }
}

#[cfg(test)]
include!("null_backend.tests.rs");
