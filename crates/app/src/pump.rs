//! Moves mail between the backend and the screen.
//!
//! The backend answers commands and also speaks on its own (a connection
//! coming up, new mail). The screen decides what it needs from what is
//! selected. Each round sends what the screen asks for, then takes what the
//! backend has to say and applies it; it stops when a round is quiet. The
//! commands go first: a switch of account has already been made on screen, so
//! its command must reach the backend before the backend is asked what is new,
//! or the account that was left is still the active one and its queued page
//! would be shown as the page of the account just chosen.

use model::MailBackend;
use ui::AppView;

/// A backend that keeps answering with more to ask for cannot loop the
/// screen forever.
const MAX_ROUNDS: usize = 16;

/// Runs rounds until quiet and returns how many events were applied.
pub fn pump(view: &mut AppView, backend: &mut dyn MailBackend) -> usize {
    let mut applied = 0;
    for _ in 0..MAX_ROUNDS {
        let mut events = Vec::new();
        for command in view.take_commands() {
            events.extend(backend.handle(command));
        }
        events.extend(backend.poll_events());
        if events.is_empty() {
            break;
        }
        applied += events.len();
        for event in events {
            view.apply_event(event);
        }
    }
    applied
}

#[cfg(test)]
include!("pump.tests.rs");
