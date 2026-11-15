//! One conversation with the mail server: the greeting, EHLO, STARTTLS and
//! the goodbye. The steps that carry a message are in `envelope.rs`, signing
//! in is in `auth.rs`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use super::caps::ServerCaps;
use super::connect::{self, HELLO_NAME, Opened};
use super::error::{Said, SmtpError, Step, from_wire};
use super::reply::Reply;
use super::timeouts::SmtpTimeouts;
use super::wire::{Wire, WireError};
use crate::account_smtp::SmtpSettings;

/// How long the goodbye may take; the connection is dropped either way.
const GOODBYE_WAIT: Duration = Duration::from_millis(1500);

/// How long a goodbye that nobody needs to hear answered may be written.
const FAREWELL_WAIT: Duration = Duration::from_millis(300);

/// Whether the message has begun to go out. Shared with the caller, which
/// needs it to say what a timeout of the whole send means.
#[derive(Debug, Clone, Default)]
pub struct Progress(Arc<AtomicBool>);

impl Progress {
    pub fn begin_message(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// The server gave a definite answer; the message is not in doubt.
    pub fn settled(&self) {
        self.0.store(false, Ordering::SeqCst);
    }

    pub fn message_begun(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// A connection and what is known about it.
pub struct Link {
    pub(super) wire: Wire,
    pub(super) limits: SmtpTimeouts,
    pub(super) progress: Progress,
    pub(super) secure: bool,
    /// The password, and whatever else was sent that is a secret, kept only
    /// to remove it from anything the server says.
    pub(super) secrets: Vec<String>,
}

impl Link {
    pub fn new(opened: Opened, settings: &SmtpSettings, limits: SmtpTimeouts, progress: Progress) -> Self {
        Self {
            wire: opened.wire,
            limits,
            progress,
            secure: opened.secure,
            secrets: vec![settings.password.expose().to_owned()],
        }
    }

    /// A wire failure as the send's failure: a doubt about the message once
    /// it has begun to go out, otherwise a failure of the step.
    pub(super) fn lost(&self, error: WireError, step: Step) -> SmtpError {
        if self.progress.message_begun() {
            SmtpError::MayHaveBeenSent
        } else {
            from_wire(error, step)
        }
    }

    pub(super) fn said(&self, reply: &Reply) -> Said {
        Said::of(reply, &self.secrets)
    }

    /// Sends one command line and reads its answer.
    pub(super) async fn ask(&mut self, line: &str, step: Step) -> Result<Reply, SmtpError> {
        let limit = self.limits.reply;
        self.wire.send_line(line, limit).await.map_err(|e| self.lost(e, step))?;
        self.wire.read_reply(limit).await.map_err(|e| self.lost(e, step))
    }

    /// Reads the server's greeting: 220 or a refusal.
    pub async fn greeting(&mut self) -> Result<(), SmtpError> {
        let limit = self.limits.connect;
        let reply = self.wire.read_reply(limit).await.map_err(|e| self.lost(e, Step::Greeting))?;
        if reply.code == 220 { Ok(()) } else { Err(SmtpError::Declined(self.said(&reply))) }
    }

    /// EHLO, and what the server offers.
    pub async fn hello(&mut self) -> Result<ServerCaps, SmtpError> {
        let line = format!("EHLO {HELLO_NAME}");
        let reply = self.ask(&line, Step::Hello).await?;
        if reply.is_positive() {
            Ok(ServerCaps::from_ehlo(&reply))
        } else {
            Err(SmtpError::Declined(self.said(&reply)))
        }
    }

    /// STARTTLS and the handshake. The old connection is spent: the new one
    /// has not said hello yet and knows nothing of the old capabilities.
    pub async fn start_tls(
        mut self,
        settings: &SmtpSettings,
        host: &str,
        caps: &ServerCaps,
    ) -> Result<Self, SmtpError> {
        if !caps.starttls {
            return Err(SmtpError::StartTlsMissing);
        }
        let reply = self.ask("STARTTLS", Step::StartTls).await?;
        if reply.code != 220 {
            return Err(SmtpError::StartTlsMissing);
        }
        let Self { wire, limits, progress, secrets, .. } = self;
        let wire = connect::start_tls(settings, host, wire, limits.connect).await?;
        Ok(Self { wire, limits, progress, secure: true, secrets })
    }

    /// Says goodbye and waits a moment for the answer; never fails. For a
    /// message that was accepted, once the person has been told.
    pub async fn quit(&mut self) {
        if self.wire.send_line("QUIT", GOODBYE_WAIT).await.is_ok() {
            let _ = self.wire.read_reply(GOODBYE_WAIT).await;
        }
    }

    /// Says goodbye without waiting for the answer, so a failure is reported
    /// at once; never fails.
    pub async fn quit_unheard(&mut self) {
        let _ = self.wire.send_line("QUIT", FAREWELL_WAIT).await;
    }
}
