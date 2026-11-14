//! The real mail backend: reads a mailbox over IMAP and, only when the
//! account says so (`read_only = false`), changes mail on the server: flags,
//! moves and deletes. A read-only account never sends anything that could
//! change the server.
//!
//! It owns one worker thread running a single-threaded async runtime.
//! Commands travel to the worker over a channel; events come back over
//! another, and `poll_events` drains it without blocking. `handle` returns at
//! once: a command that needs the server is answered through `poll_events`
//! later, and one that is not available (or a change asked for while the
//! connection is down) is answered on the spot.
//!
//! A change is sent to the server and its events are produced only after the
//! server confirmed it. A refusal or a lost connection becomes a failure
//! notice and nothing is changed locally; nothing is queued for later.

mod backoff;
mod change_flags;
mod change_relocate;
mod change_rules;
mod changes;
mod handlers;
mod host_info;
mod lifecycle;
mod offline;
mod online;
mod resolve;
mod sink;
mod stop;
mod timing;
mod unsupported;
mod worker;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread::JoinHandle;

use model::{AccountId, Command, ConnectionState, Event, MailBackend};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio::sync::watch;

use self::lifecycle::connection;
use self::sink::EventSink;
use self::stop::Stop;
use self::unsupported::{
    OFFLINE_CHANGE_REASON, change_operation, unavailable_operation, unavailable_reason,
};
use self::worker::Worker;
use crate::account_config::AccountConfig;

#[cfg(feature = "test-support")]
pub use self::timing::Timing;
#[cfg(not(feature = "test-support"))]
use self::timing::Timing;

/// A backend that serves one account from a real IMAP server. It is
/// read-only unless the account file allows changes.
pub struct RealBackend {
    account: AccountId,
    /// Whether the account may change mail on the server.
    writes: bool,
    /// True while the worker's connection is up.
    online: Arc<AtomicBool>,
    commands: Option<UnboundedSender<Command>>,
    events: Receiver<Event>,
    stop: watch::Sender<bool>,
    worker: Option<JoinHandle<()>>,
}

impl RealBackend {
    /// Starts the worker and begins connecting. Progress, folders and mail
    /// arrive through `poll_events`.
    pub fn start(config: AccountConfig) -> Self {
        Self::start_timed(config, Timing::production())
    }

    /// Like `start`, with the waits and limits chosen by the caller. For
    /// tests only: the app never turns this feature on.
    #[cfg(feature = "test-support")]
    pub fn start_with_timing(config: AccountConfig, timing: Timing) -> Self {
        Self::start_timed(config, timing)
    }

    fn start_timed(config: AccountConfig, timing: Timing) -> Self {
        let account = AccountId(config.id.clone());
        let (event_sender, events) = mpsc::channel();
        let (command_sender, command_receiver) = unbounded_channel();
        let (stop_sender, stop) = Stop::pair();
        let sink = EventSink::new(event_sender, account.clone());
        let online = sink.online_flag();
        let writes = !config.read_only;
        let failed_sink = sink.clone();
        let spawned = std::thread::Builder::new()
            .name("mail-worker".to_owned())
            .spawn(move || run_worker(Worker::new(config, timing, sink, stop), command_receiver));
        let worker = match spawned {
            Ok(handle) => Some(handle),
            Err(error) => {
                failed_sink.send(failed_sink.failure("start", format!("no worker thread: {error}")));
                None
            }
        };
        Self {
            account,
            writes,
            online,
            commands: worker.is_some().then_some(command_sender),
            events,
            stop: stop_sender,
            worker,
        }
    }

    fn unavailable(&self, operation: &str, reason: &str) -> Vec<Event> {
        vec![Event::OpFailed {
            account: Some(self.account.clone()),
            operation: operation.to_owned(),
            reason: reason.to_owned(),
        }]
    }
}

/// The worker thread's body: a runtime that lives as long as the supervisor.
/// If the supervisor ever panics, the screen is told the account went offline
/// instead of staying "online" forever.
fn run_worker(worker: Worker, mut commands: tokio::sync::mpsc::UnboundedReceiver<Command>) {
    let sink = worker.sink.clone();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    let runtime = match runtime {
        Ok(runtime) => runtime,
        Err(error) => {
            sink.send(sink.failure("start", format!("no runtime: {error}")));
            return;
        }
    };
    let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.block_on(worker.run(&mut commands));
    }));
    if ran.is_err() {
        sink.send(connection(&sink, ConnectionState::Offline));
        sink.send(sink.failure("worker", "the mail worker stopped unexpectedly"));
    }
    // A name lookup still running in the background must not hold up
    // shutting down.
    runtime.shutdown_background();
}

impl MailBackend for RealBackend {
    fn handle(&mut self, command: Command) -> Vec<Event> {
        if let Some(operation) = unavailable_operation(&command, self.writes) {
            return self.unavailable(operation, unavailable_reason(self.writes));
        }
        if let Some(operation) = change_operation(&command)
            && !self.online.load(Ordering::Acquire)
        {
            return self.unavailable(operation, OFFLINE_CHANGE_REASON);
        }
        let sent = self
            .commands
            .as_ref()
            .is_some_and(|channel| channel.send(command).is_ok());
        if sent {
            Vec::new()
        } else {
            self.unavailable("command", "the mail worker is not running")
        }
    }

    fn poll_events(&mut self) -> Vec<Event> {
        self.events.try_iter().collect()
    }
}

impl Drop for RealBackend {
    /// Stops the worker and waits for it. Every wait in the worker gives way
    /// to the stop signal, so this returns promptly.
    fn drop(&mut self) {
        let _ = self.stop.send(true);
        drop(self.commands.take());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
