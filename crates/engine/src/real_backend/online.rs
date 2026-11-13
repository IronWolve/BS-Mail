//! Serving commands over a live session, with a periodic check that the
//! connection is still there.

use model::{Command, ConnectionState, Event};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::{Instant, MissedTickBehavior, interval_at};

use super::lifecycle::connection;
use super::worker::Worker;
use crate::imap_session::{ImapSession, SessionError};

/// Why serving ended.
pub(super) enum Served {
    /// The owner asked the worker to finish.
    Stopped,
    /// The connection is gone; try again later.
    Lost,
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
        let period = self.timing.keepalive;
        let mut keepalive = interval_at(Instant::now() + period, period);
        keepalive.set_missed_tick_behavior(MissedTickBehavior::Delay);
        match stop.guard(self.go_online(session)).await {
            None => return Served::Stopped,
            Some(Err(_)) => return Served::Lost,
            Some(Ok(())) => {}
        }
        loop {
            tokio::select! {
                biased;
                () = stop.requested() => return Served::Stopped,
                received = commands.recv() => {
                    let Some(command) = received else {
                        return Served::Stopped;
                    };
                    match stop.guard(self.dispatch(session, command)).await {
                        None => return Served::Stopped,
                        Some(Err(_)) => return Served::Lost,
                        Some(Ok(())) => {}
                    }
                }
                _ = keepalive.tick() => {
                    match stop.guard(session.ping()).await {
                        None => return Served::Stopped,
                        Some(Err(error)) if error.is_lost() => return Served::Lost,
                        Some(_) => {}
                    }
                }
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
        Ok(())
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
            Command::FetchBody { message } => {
                let body = self.fetch_body(session, message).await;
                self.settle("fetch body", body)
            }
            Command::SyncNow { account } => {
                let synced = self.sync_now(session, &account).await;
                self.settle("sync", synced)
            }
            other => {
                self.answer_offline(other);
                Ok(())
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
