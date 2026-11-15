//! The account being tried: its own backend, and what it says.
//!
//! The form shows only what it needs of that (that the account is in, or why
//! it is not); everything it says is kept in order so the account joins the
//! running ones with its history.

use model::{ConnectionState, Event, MailBackend};
use ui::AppView;

use crate::accounts::Plan;

use super::attempt::{Attempt, Stage};
use super::flow::SetupFlow;

/// How a poll of the account being tried ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Polled {
    /// Nothing decisive yet.
    Pending,
    /// The server let the account in.
    Online,
    /// The account was refused or could not connect; the form has the reason.
    Failed,
}

impl SetupFlow {
    /// Starts the account on a backend of its own. The old attempt, if any,
    /// is already gone.
    pub(super) fn start_trial(&mut self, plan: Plan) {
        let config = plan.config.clone().with_state_dir(&plan.folder);
        let backend = self.backends.start(config);
        self.attempt = Some(Attempt { plan, stage: Stage::Trial { backend, kept: Vec::new() } });
    }

    /// Reads what the account being tried has said. When the server has let
    /// it in, it is saved and joins the running accounts; when it was
    /// refused, the attempt ends and the form shows why.
    pub(super) fn watch_trial(&mut self, view: &mut AppView, backend: &mut Box<dyn MailBackend>) -> bool {
        let Some(Attempt { stage: Stage::Trial { backend: trial, kept }, .. }) = self.attempt.as_mut() else {
            return false;
        };
        let before = kept.len();
        let polled = poll_trial(trial.as_mut(), kept, view);
        let heard = kept.len() != before;
        match polled {
            Polled::Pending => heard,
            Polled::Online => {
                self.complete(view, backend);
                true
            }
            Polled::Failed => {
                self.attempt = None;
                self.restore_token();
                true
            }
        }
    }
}

/// Polls the trial once, keeping every event, and lets the form read the ones
/// that are about it. An event that refuses the account ends the poll: the
/// attempt is over and the rest is not wanted.
pub(super) fn poll_trial(trial: &mut dyn MailBackend, kept: &mut Vec<Event>, view: &mut AppView) -> Polled {
    let mut online = false;
    for event in trial.poll_events() {
        let failed = matches!(event, Event::OpFailed { .. });
        let signed_in = matches!(event, Event::ConnectionState { state: ConnectionState::Online, .. });
        if failed || signed_in {
            view.note_setup_event(&event);
        }
        kept.push(event);
        online |= signed_in;
        if failed && !online {
            return Polled::Failed;
        }
    }
    // The flag the form raised is the flow's own: take it so it never lingers.
    let _ = view.take_setup_online();
    if online { Polled::Online } else { Polled::Pending }
}
