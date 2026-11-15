//! The worker thread's supervisor: connect, serve commands while connected,
//! and when the connection is lost or refused, wait and try again.

use std::time::{Duration, Instant};

use model::{AccountId, Command, ConnectionState, FolderId, MessageId};
use tokio::sync::mpsc::UnboundedReceiver;

use super::backoff::Backoff;
use super::lifecycle::{announce, changes_allowed, connection};
use super::in_flight::SendsInFlight;
use super::inbox::Inbox;
use super::online::Served;
use super::poll_state::PollState;
use super::retry_ids::RetryIds;
use super::save_sent::{SentCopies, SentCopy};
use super::sent_queue::SentQueue;
use super::sink::EventSink;
use super::stop::Stop;
use super::thread_cache::ThreadCache;
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
    /// How many times the connection was lost while that body was coming.
    pub body_losses: u8,
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
    /// What the automatic check for new mail remembers.
    pub(super) poll: PollState,
    /// True while the screen is showing the answer to a search, which the
    /// next page of the open folder would end.
    pub(super) search_shown: bool,
    /// Sent messages whose copies are waiting to be saved in the Sent folder.
    pub(super) copies: SentCopies,
    /// Sent copies waiting for the mailbox: made while it was out of reach,
    /// or left on disk by an earlier run.
    pub(super) unsaved: Vec<SentCopy>,
    /// Where those copies are kept on disk.
    pub(super) queue: SentQueue,
    /// The message ids of sends that did not end well.
    pub(super) retry_ids: RetryIds,
    /// The thread ids of the messages opened lately, for replies to them.
    pub(super) threads: ThreadCache,
    /// Originals of replies the server accepted, reported by the send tasks.
    pub(super) answers: Inbox<MessageId>,
    /// Originals still to be marked as answered, waiting for the mailbox.
    pub(super) answered_waiting: Vec<MessageId>,
    /// The sends on their way, which closing waits for.
    pub(super) sends: SendsInFlight,
    /// The person was told, in this run, that no Sent folder is recognized.
    pub(super) no_sent_told: bool,
}

/// How a pause between connection attempts ended.
enum Pause {
    Stopped,
    Retry,
}

impl Worker {
    pub(super) fn new(
        config: AccountConfig,
        timing: Timing,
        sink: EventSink,
        stop: Stop,
        sends: SendsInFlight,
    ) -> Self {
        let queue = SentQueue::for_account(&config);
        Self {
            account: sink.account().clone(),
            config,
            timing,
            sink,
            stop,
            state: SessionState::new(),
            wanted: Wanted::default(),
            last_sync: None,
            poll: PollState::default(),
            search_shown: false,
            copies: SentCopies::new(),
            unsaved: Vec::new(),
            queue,
            retry_ids: RetryIds::default(),
            threads: ThreadCache::default(),
            answers: Inbox::new(),
            answered_waiting: Vec::new(),
            sends,
            no_sent_told: false,
        }
    }

    /// Runs until stopped.
    pub(super) async fn run(mut self, commands: &mut UnboundedReceiver<Command>) {
        announce(&self.config, &self.sink);
        self.unsaved = self.queue.waiting();
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
                    told_user = false;
                    let began = Instant::now();
                    match self.serve(session, commands).await {
                        Served::Stopped => return,
                        Served::Lost => {
                            // Signing in is not enough: a server that accepts the
                            // sign-in and then drops the session is still failing.
                            if began.elapsed() >= self.timing.stable_after {
                                backoff.reset();
                            }
                            Some(backoff.next_wait())
                        }
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
        if permanent { None } else { Some(backoff.next_wait()) }
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
                copy = self.copies.recv() => {
                    if let Some(copy) = copy {
                        self.sent_copy_offline(copy);
                    }
                }
                original = self.answers.recv() => {
                    if let Some(original) = original {
                        self.answered_offline(original);
                    }
                }
            }
        }
    }
}
