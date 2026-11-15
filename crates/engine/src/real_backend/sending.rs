//! Carrying out a Send.
//!
//! The message is checked first, with no network at all. A reply names the
//! `Message-ID` and `References` of its original, which were kept when the
//! original was opened; only for an original that was not opened lately, and
//! only while the mailbox connection is up, are they read from the server.
//! When the server cannot give them, the ids the list showed for the original
//! stand in (see `reply_lookup`). A reply whose original cannot be found at
//! all goes out without them, and a notice named `reply thread` says so. The
//! send itself then runs on a task of its own, with its own connection.
//!
//! Exactly one event answers a send: `MessageSent` when the server's final
//! answer to DATA was a success, or a failure notice named `send`. A send is
//! never repeated: nothing here keeps the message once the task has it, so
//! no reconnect can replay it. The saved copy is handed back to the worker
//! only after `MessageSent` has been sent. The original of a reply is told to
//! the worker after that too, so it can be marked as answered.

use model::{Event, MessageId, OutgoingMessage};
use tokio::sync::mpsc::UnboundedSender;

use super::in_flight::InFlight;
use super::reply_lookup::{LOST, Lookup, OFFLINE};
use super::save_sent::SentCopy;
use super::retry_ids::RetryIds;
use super::sent_queue::SentQueue;
use super::sink::EventSink;
use super::stop::Stop;
use super::unsupported::SEND_READ_ONLY_REASON;
use super::worker::Worker;
use crate::account_smtp::{SmtpAuth, SmtpSettings};
use crate::imap_session::{ImapSession, SessionError};
use crate::oauth::{HttpLimits, TokenSource};
use crate::send::{Checked, Prepared, Sender, SmtpError, SmtpTimeouts, check, now_unix, submit};

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
    /// Told when the server accepts the message.
    retry_ids: RetryIds,
    /// Told the original of a reply, to be marked as answered, when the
    /// server accepts it.
    answers: UnboundedSender<MessageId>,
    original: Option<MessageId>,
    /// Counts this send as on its way until the task ends, so that closing
    /// the program waits for it.
    _in_flight: InFlight,
    /// Which message this is, for `retry_ids`.
    key: u64,
}

/// What is said when the server took the message and its copy could not be
/// written to disk: the copy is held in memory only.
const COPY_NOT_ON_DISK: &str = "the message was sent, but its copy could not be written to disk, so it is held \
     in memory until the Sent folder has it; if the program is closed first, the copy is lost";

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
        let mut lookup = Lookup::default();
        if let Some(original) = message.in_reply_to {
            lookup = match self.reply_ids(session, original).await {
                Ok(found) => found,
                Err(error) => {
                    // The connection failed while asking: what the list showed
                    // is the next best thing, and the loss is still handed on.
                    lost = Some(error).filter(SessionError::is_lost);
                    self.listed_lookup(original, LOST)
                }
            };
        }
        self.launch(checked, &lookup, message.in_reply_to);
        lost.map_or(Ok(()), Err)
    }

    /// Serves a Send while the mailbox connection is down.
    pub(super) fn send_offline(&self, message: OutgoingMessage) {
        if let Some(checked) = self.accept(&message) {
            let lookup = message.in_reply_to.map(|original| self.known_ids(original, OFFLINE)).unwrap_or_default();
            self.launch(checked, &lookup, message.in_reply_to);
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

    /// Dates and numbers the message and starts the task that sends it. A
    /// reply that goes out without thread headers, because its original could
    /// not be read, says so in a notice of its own.
    fn launch(&self, checked: Checked, lookup: &Lookup, original: Option<MessageId>) {
        let key = checked.fingerprint();
        let earlier = self.retry_ids.earlier(key);
        let prepared = match checked.finish_as(lookup.ids.as_ref(), now_unix(), earlier) {
            Ok(prepared) => prepared,
            Err(refusal) => {
                self.sink.send(failure(&self.sink, refusal.reason()));
                return;
            }
        };
        self.retry_ids.begin(key, prepared.message_id());
        if let Some(why) = lookup.missing.filter(|_| original.is_some()) {
            self.sink.send(self.sink.failure("reply thread", format!(
                "this reply goes out without In-Reply-To and References, so its recipients may see \
                 it as a new conversation: {why}"
            )));
        }
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
            retry_ids: self.retry_ids.clone(),
            answers: self.answers.sender(),
            original,
            _in_flight: self.sends.begin(),
            key,
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
            job.retry_ids.accepted(job.key);
            // The copy is written before the send is called a success, so
            // that nothing which says "sent" leaves the only record of it in
            // memory for even a moment.
            let copy = job.settings.save_sent.then(|| {
                let bytes = prepared.sent_copy(&accepted.take_bytes());
                let file = job.queue.store(&bytes);
                SentCopy { bytes, file, uncertain: false }
            });
            job.sink.send(Event::MessageSent { account: job.sink.account().clone() });
            if let Some(copy) = copy {
                if copy.file.is_none() && job.queue.keeps_files() {
                    job.sink.send(job.sink.failure("save sent copy", COPY_NOT_ON_DISK));
                }
                let _ = job.copies.send(copy);
            }
            if let Some(original) = job.original {
                let _ = job.answers.send(original);
            }
            job.stop.guard(accepted.say_goodbye()).await;
        }
        Some(Err(error)) => job.sink.send(failure(&job.sink, error.reason())),
    }
}
