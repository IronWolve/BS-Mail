//! Carrying out a Send.
//!
//! The message is checked first, with no network at all. When the mailbox
//! connection is up and the message is a reply, the one thing read from it
//! is the original's `Message-ID` and `References`, so the reply can name
//! them; a reply whose original cannot be found goes out without them. The
//! send itself then runs on a task of its own, with its own connection.
//!
//! Exactly one event answers a send: `MessageSent` when the server's final
//! answer to DATA was a success, or a failure notice named `send`. A send is
//! never repeated: nothing here keeps the message once the task has it, so
//! no reconnect can replay it. The saved copy is handed back to the worker
//! only after `MessageSent` has been sent.

use model::{Event, OutgoingMessage};
use tokio::sync::mpsc::UnboundedSender;

use super::save_sent::SentCopy;
use super::sent_queue::SentQueue;
use super::sink::EventSink;
use super::stop::Stop;
use super::unsupported::SEND_READ_ONLY_REASON;
use super::worker::Worker;
use crate::account_smtp::{SmtpAuth, SmtpSettings};
use crate::imap_session::{ImapSession, SessionError};
use crate::oauth::{HttpLimits, TokenSource};
use crate::send::{
    Checked, Prepared, ReplyIds, Sender, SmtpError, SmtpTimeouts, check, now_unix, submit,
};

/// What a send task needs, all of it owned.
struct SendJob {
    settings: SmtpSettings,
    /// Gives the access token when the submission server is signed in to
    /// with one.
    tokens: Option<TokenSource>,
    limits: SmtpTimeouts,
    sink: EventSink,
    stop: Stop,
    copies: UnboundedSender<SentCopy>,
    /// Where the copy is kept on disk until the mailbox has it.
    queue: SentQueue,
}

/// The failure notice for a send, named as the screen expects.
fn failure(sink: &EventSink, reason: impl Into<String>) -> Event {
    sink.failure("send", reason)
}

impl Worker {
    /// Serves a Send while the mailbox connection is up. Only a lost
    /// connection is an error here, and the send has gone ahead anyway.
    pub(super) async fn send_online(
        &mut self,
        session: &mut ImapSession,
        message: OutgoingMessage,
    ) -> Result<(), SessionError> {
        let Some(checked) = self.accept(&message) else {
            return Ok(());
        };
        let mut lost = None;
        let mut ids = None;
        if let Some(original) = message.in_reply_to {
            match self.reply_ids(session, original).await {
                Ok(found) => ids = found,
                Err(error) if error.is_lost() => lost = Some(error),
                Err(_) => {}
            }
        }
        self.launch(checked, ids.as_ref());
        lost.map_or(Ok(()), Err)
    }

    /// Serves a Send while the mailbox connection is down.
    pub(super) fn send_offline(&self, message: OutgoingMessage) {
        if let Some(checked) = self.accept(&message) {
            self.launch(checked, None);
        }
    }

    /// The message if it may go: this account's, from an account that can
    /// send, and well formed. Otherwise the reason is sent and `None` comes
    /// back.
    fn accept(&self, message: &OutgoingMessage) -> Option<Checked> {
        let refuse = |reason: &str| {
            self.sink.send(failure(&self.sink, reason));
            None
        };
        if self.config.read_only {
            return refuse(SEND_READ_ONLY_REASON);
        }
        if message.account != self.account {
            return refuse("this message is for another account; nothing was sent");
        }
        if self.config.smtp.host.is_none() {
            return refuse(&SmtpError::NoHost.reason());
        }
        let sender = Sender { name: &self.config.name, address: &self.config.address };
        match check(sender, message) {
            Ok(checked) => Some(checked),
            Err(refusal) => refuse(refusal.reason()),
        }
    }

    /// Dates and numbers the message and starts the task that sends it.
    fn launch(&self, checked: Checked, ids: Option<&ReplyIds>) {
        let prepared = match checked.finish(ids, now_unix()) {
            Ok(prepared) => prepared,
            Err(refusal) => {
                self.sink.send(failure(&self.sink, refusal.reason()));
                return;
            }
        };
        let tokens = match self.config.smtp.auth {
            SmtpAuth::OAuth2 => TokenSource::with_limits(&self.config, HttpLimits::within(self.timing.smtp.connect)),
            SmtpAuth::Password => None,
        };
        let job = SendJob {
            settings: self.config.smtp.clone(),
            tokens,
            limits: self.timing.smtp,
            sink: self.sink.clone(),
            stop: self.stop.clone(),
            copies: self.copies.sender(),
            queue: self.queue.clone(),
        };
        tokio::spawn(watched(job, prepared));
    }
}

/// Runs the send on a task of its own and answers for it should that task
/// die unexpectedly, so the composer is never left waiting.
async fn watched(job: SendJob, prepared: Prepared) {
    let sink = job.sink.clone();
    if let Err(error) = tokio::spawn(deliver(job, prepared)).await
        && error.is_panic()
    {
        sink.send(failure(&sink, "the send stopped unexpectedly, so the message may or may not \
             have been sent: check your Sent folder before trying again"));
    }
}

async fn deliver(job: SendJob, prepared: Prepared) {
    let sent = job.stop.guard(submit(&job.settings, job.tokens.as_ref(), job.limits, &prepared)).await;
    match sent {
        None => {}
        Some(Ok(mut accepted)) => {
            job.sink.send(Event::MessageSent { account: job.sink.account().clone() });
            if job.settings.save_sent {
                let bytes = accepted.take_bytes();
                let file = job.queue.store(&bytes);
                let _ = job.copies.send(SentCopy { bytes, file });
            }
            job.stop.guard(accepted.say_goodbye()).await;
        }
        Some(Err(error)) => job.sink.send(failure(&job.sink, error.reason())),
    }
}
