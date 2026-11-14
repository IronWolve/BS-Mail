//! The worker thread's supervisor: connect, serve commands while connected,
//! and when the connection is lost or refused, wait and try again.

use std::time::Duration;

use model::{AccountId, Command, ConnectionState, FolderId, MessageId};
use tokio::sync::mpsc::UnboundedReceiver;

use super::backoff::Backoff;
use super::lifecycle::{announce, changes_allowed, connection};
use super::online::Served;
use super::sink::EventSink;
use super::stop::Stop;
use super::timing::Timing;
use crate::account_config::AccountConfig;
use crate::imap_session::{ConnectError, ImapSession};
use crate::session_state::SessionState;

/// What to serve again once the connection is back: the open folder, and a
/// body whose fetch did not finish.
#[derive(Debug, Default)]
pub(super) struct Wanted {
    pub folder: Option<FolderId>,
    pub body: Option<MessageId>,
}

pub(super) struct Worker {
    pub config: AccountConfig,
    pub account: AccountId,
    pub timing: Timing,
    pub sink: EventSink,
    pub stop: Stop,
    pub state: SessionState,
    pub wanted: Wanted,
    pub last_sync: Option<i64>,
}

/// How a pause between connection attempts ended.
enum Pause {
    Stopped,
    Retry,
}

impl Worker {
    pub(super) fn new(config: AccountConfig, timing: Timing, sink: EventSink, stop: Stop) -> Self {
        Self {
            account: sink.account().clone(),
            config,
            timing,
            sink,
            stop,
            state: SessionState::new(),
            wanted: Wanted::default(),
            last_sync: None,
        }
    }

    /// Runs until stopped.
    pub(super) async fn run(mut self, commands: &mut UnboundedReceiver<Command>) {
        announce(&self.config, &self.sink);
        let mut backoff = Backoff::new(self.timing.backoff_start, self.timing.backoff_max);
        let mut told_user = false;
        let mut first = true;
        loop {
            if !std::mem::take(&mut first) {
                self.sink.send(connection(&self.sink, ConnectionState::Connecting));
                self.sink.send(changes_allowed(&self.config, &self.sink));
            }
            let attempt = ImapSession::connect(&self.config, self.timing.timeouts);
            let Some(outcome) = self.stop.guard(attempt).await else {
                return;
            };
            let pause = match outcome {
                Ok(session) => {
                    backoff.reset();
                    told_user = false;
                    match self.serve(session, commands).await {
                        Served::Stopped => return,
                        Served::Lost => Some(backoff.next_delay()),
                    }
                }
                Err(error) => self.after_failed_connect(&error, &mut told_user, &mut backoff),
            };
            self.sink.send(connection(&self.sink, ConnectionState::Offline));
            match self.pause(pause, commands).await {
                Pause::Stopped => return,
                Pause::Retry => {}
            }
        }
    }

    /// Reports a failed attempt and says how long to wait: a retry delay for
    /// trouble that may pass, `None` (wait for the user) for a bad password
    /// or certificate.
    fn after_failed_connect(
        &self,
        error: &ConnectError,
        told_user: &mut bool,
        backoff: &mut Backoff,
    ) -> Option<Duration> {
        let permanent = error.is_permanent();
        if permanent || !*told_user {
            self.sink.send(self.sink.failure(error.operation(), error.reason()));
            *told_user = true;
        }
        if permanent { None } else { Some(backoff.next_delay()) }
    }

    /// Waits out the delay (or, with none, waits for the user to ask for a
    /// sync) while answering commands that cannot be served offline.
    async fn pause(
        &mut self,
        delay: Option<Duration>,
        commands: &mut UnboundedReceiver<Command>,
    ) -> Pause {
        let stop = self.stop.clone();
        let deadline = delay.map(|d| tokio::time::Instant::now() + d);
        loop {
            let tick = async {
                match deadline {
                    Some(at) => tokio::time::sleep_until(at).await,
                    None => std::future::pending().await,
                }
            };
            tokio::select! {
                biased;
                () = stop.requested() => return Pause::Stopped,
                () = tick => return Pause::Retry,
                received = commands.recv() => match received {
                    None => return Pause::Stopped,
                    Some(Command::SyncNow { .. }) => return Pause::Retry,
                    Some(other) => self.answer_offline(other),
                },
            }
        }
    }
}
