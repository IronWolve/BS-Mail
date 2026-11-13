//! The one interface between the UI and any mail engine, real or mock.

use crate::command::Command;
use crate::event::Event;

pub trait MailBackend: Send {
    /// Accepts one command and returns the events that are ready right away.
    /// Slow work answers later through `poll_events`.
    fn handle(&mut self, command: Command) -> Vec<Event>;

    /// Drains events that arrived on their own since the last call: new mail,
    /// connection changes, finished fetches. A backend with none returns an
    /// empty list.
    fn poll_events(&mut self) -> Vec<Event> {
        Vec::new()
    }
}
