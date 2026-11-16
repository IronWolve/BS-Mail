//! Serving commands over a live session, with a periodic check that the
//! connection is still there and, unless it is switched off, a periodic look
//! for new mail. Everything runs one thing at a time: a timer that comes due
//! while a command is running waits for it to finish.

use std::time::Duration;

use model::{Command, ConnectionState, Event};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::{Instant, MissedTickBehavior, interval_at};

use super::lifecycle::connection;
use super::raw_views::operation_of;
use super::unsupported::change_operation;
use super::worker::Worker;
use crate::imap_session::{ImapSession, SessionError};

/// Why serving ended.
pub(super) enum Served {
    /// The owner asked the worker to finish.
    Stopped,
    /// The connection is gone; try again later.
    Lost,
}

/// What a finished step means for serving: keep going, or end the session
/// for the reason it gives. `None` from the guard means stopping was asked.
fn verdict(step: Option<Result<(), SessionError>>) -> Option<Served> {
    match step {
        None => Some(Served::Stopped),
        Some(Err(_)) => Some(Served::Lost),
        Some(Ok(())) => None,
    }
}

/// A timer that ticks every `period`, first after one whole period.
fn ticker(period: Duration) -> tokio::time::Interval {
    let mut timer = interval_at(Instant::now() + period, period);
    timer.set_missed_tick_behavior(MissedTickBehavior::Delay);
    timer
}

impl Worker {
    /// Serves one live session, then says goodbye if the worker is stopping.
    pub(super) async fn serve(
        &mut self,
        mut session: ImapSession,
        commands: &mut UnboundedReceiver<Command>,
    ) -> Served {
        let outcome = self.serve_session(&mut session, commands).await;
        if matches!(outcome, Served::Stopped) {
            session.close().await;
        }
        outcome
    }

    /// Announces the session, sends the folder tree and status, replays
    /// what was wanted before the connection dropped, then serves commands.
    async fn serve_session(
        &mut self,
        session: &mut ImapSession,
        commands: &mut UnboundedReceiver<Command>,
    ) -> Served {
        let stop = self.stop.clone();
        let mut keepalive = ticker(self.timing.keepalive);
        let poll_period = self.poll_period();
        let mut poll = ticker(poll_period.unwrap_or(Duration::from_secs(3600)));
        let polling = poll_period.is_some();
        if let Some(done) = verdict(stop.guard(self.go_online(session)).await) {
            return done;
        }
        loop {
            let ended = tokio::select! {
                biased;
                () = stop.requested() => Some(Served::Stopped),
                received = commands.recv() => match received {
                    None => Some(Served::Stopped),
                    Some(command) => verdict(stop.guard(self.dispatch(session, command)).await),
                },
                copy = self.copies.recv() => match copy {
                    None => None,
                    Some(copy) => verdict(stop.guard(self.save_sent(session, copy)).await),
                },
                original = self.answers.recv() => match original {
                    None => None,
                    Some(original) => verdict(stop.guard(self.mark_answered(session, original)).await),
                },
                _ = keepalive.tick() => match stop.guard(session.ping()).await {
                    None => Some(Served::Stopped),
                    Some(Err(error)) if error.is_lost() => Some(Served::Lost),
                    Some(_) => None,
                },
                _ = poll.tick(), if polling => verdict(stop.guard(self.poll_check(session)).await),
            };
            if let Some(done) = ended {
                return done;
            }
        }
    }

    /// The events of a fresh session, then whatever was asked for while the
    /// connection was down.
    async fn go_online(&mut self, session: &mut ImapSession) -> Result<(), SessionError> {
        self.sink.send(connection(&self.sink, ConnectionState::Online));
        let refreshed = self.refresh(session).await;
        self.settle("sync", refreshed)?;
        if let Some(folder) = self.wanted.folder {
            let page = self.open_folder(session, folder).await;
            self.settle("open folder", page)?;
        }
        if let Some(message) = self.wanted.body {
            let body = self.fetch_body(session, message).await;
            self.settle("fetch body", body)?;
        }
        self.save_unsaved(session).await?;
        self.mark_waiting_answered(session).await
    }

    /// Runs one command that needs the server.
    async fn dispatch(
        &mut self,
        session: &mut ImapSession,
        command: Command,
    ) -> Result<(), SessionError> {
        match command {
            Command::OpenFolder { folder } => {
                let page = self.open_folder(session, folder).await;
                self.settle("open folder", page)
            }
            Command::LoadMore { folder, have } => {
                let page = self.load_more(session, folder, have).await;
                self.settle("load older mail", page)
            }
            Command::FetchBody { message } => {
                let body = self.fetch_body(session, message).await;
                self.settle("fetch body", body)
            }
            Command::FetchRaw { message, part } => {
                let raw = self.fetch_raw(session, message, part).await;
                self.settle(operation_of(part), raw)
            }
            Command::PrepareHtml { message } => {
                let page = self.prepare_html(session, message).await;
                self.settle("open in browser", page)
            }
            Command::SyncNow { account } => {
                let synced = self.sync_now(session, &account).await;
                self.settle("sync", synced)
            }
            Command::Search { request, account, query } => {
                let found = self.search(session, request, &account, &query).await;
                self.settle_search(found)
            }
            change if change_operation(&change).is_some() => self.change(session, change).await,
            Command::Send { message } => self.send_online(session, message).await,
            other => {
                self.answer_offline(other);
                Ok(())
            }
        }
    }

    /// Sends the answer to a search. A connection lost on the way is still
    /// answered, once: the search is not run again after the reconnect, the
    /// person asks again.
    fn settle_search(&self, result: Result<Vec<Event>, SessionError>) -> Result<(), SessionError> {
        match result {
            Ok(events) => {
                self.sink.send_all(events);
                Ok(())
            }
            Err(error) => {
                let reason = "the connection was lost during the search; ask again once it is back";
                self.sink.send(self.sink.failure("search", reason));
                Err(error)
            }
        }
    }

    /// Sends the events of a finished job. A refusal becomes a failure
    /// notice; a lost connection is handed back for the caller to act on.
    fn settle(
        &self,
        operation: &str,
        result: Result<Vec<Event>, SessionError>,
    ) -> Result<(), SessionError> {
        match result {
            Ok(events) => {
                self.sink.send_all(events);
                Ok(())
            }
            Err(error) if error.is_lost() => Err(error),
            Err(error) => {
                self.sink.send(self.sink.failure(operation, error.text()));
                Ok(())
            }
        }
    }
}
