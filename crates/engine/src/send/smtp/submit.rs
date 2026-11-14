//! One send, start to finish, on its own connection.
//!
//! The conversation is: greeting, EHLO, STARTTLS and EHLO again (when the
//! account says `starttls`), sign in, MAIL FROM, one RCPT TO for each
//! recipient, DATA, QUIT. Nothing here is ever tried a second time: a
//! failure is reported and the connection is dropped.

use super::caps::ServerCaps;
use super::connect;
use super::data::is_wire_safe;
use super::error::{SmtpError, Step};
use super::link::{Link, Progress};
use super::timeouts::SmtpTimeouts;
use crate::account_config::Security;
use crate::account_smtp::SmtpSettings;
use crate::send::check::Refusal;
use crate::send::message::Prepared;

/// A message the server accepted, and the connection it was accepted on,
/// which still owes the server a goodbye.
pub struct Accepted {
    bytes: Vec<u8>,
    link: Link,
}

impl Accepted {
    /// The exact bytes the server accepted, which is what a Sent copy must
    /// hold. Leaves nothing behind.
    pub fn take_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.bytes)
    }

    /// Ends the conversation. Not part of the send: the person has been told
    /// by now, and nothing here can change the outcome.
    pub async fn say_goodbye(mut self) {
        self.link.quit().await;
    }
}

/// Sends `prepared` through the account's server. The answer comes as soon
/// as the server has accepted the message; the goodbye is left to the caller.
pub async fn submit(
    settings: &SmtpSettings,
    limits: SmtpTimeouts,
    prepared: &Prepared,
) -> Result<Accepted, SmtpError> {
    let Some(host) = settings.host.as_deref() else {
        return Err(SmtpError::NoHost);
    };
    let progress = Progress::default();
    let whole = converse(settings, host, limits, prepared, progress.clone());
    match tokio::time::timeout(limits.total, whole).await {
        Ok(result) => result,
        Err(_) if progress.message_begun() => Err(SmtpError::MayHaveBeenSent),
        Err(_) => Err(SmtpError::TimedOut(Step::Whole)),
    }
}

async fn converse(
    settings: &SmtpSettings,
    host: &str,
    limits: SmtpTimeouts,
    prepared: &Prepared,
    progress: Progress,
) -> Result<Accepted, SmtpError> {
    let opened = connect::open(settings, host, &limits).await?;
    let mut link = Link::new(opened, settings, limits, progress);
    link.greeting().await?;
    let mut caps = link.hello().await?;
    if settings.security == Security::Starttls {
        link = link.start_tls(settings, host, &caps).await?;
        caps = link.hello().await?;
    }
    match transact(&mut link, settings, &caps, prepared).await {
        Ok(bytes) => Ok(Accepted { bytes, link }),
        Err(SmtpError::MayHaveBeenSent) => Err(SmtpError::MayHaveBeenSent),
        Err(error) => {
            link.quit_unheard().await;
            Err(error)
        }
    }
}

/// Everything between saying hello and saying goodbye.
async fn transact(
    link: &mut Link,
    settings: &SmtpSettings,
    caps: &ServerCaps,
    prepared: &Prepared,
) -> Result<Vec<u8>, SmtpError> {
    link.sign_in(settings, caps).await?;
    let rendered = prepared.render(caps.eight_bit).map_err(SmtpError::Refused)?;
    if !is_wire_safe(&rendered.bytes) {
        return Err(SmtpError::Refused(Refusal::new(
            "the message could not be built safely; nothing was sent",
        )));
    }
    if let Some(limit) = caps.size.filter(|limit| *limit > 0) {
        if u64::try_from(rendered.bytes.len()).unwrap_or(u64::MAX) > limit {
            return Err(SmtpError::TooBig(Some(limit)));
        }
    }
    let size = caps.size.map(|_| rendered.bytes.len());
    link.mail_from(prepared.sender(), size, rendered.eight_bit, caps.size).await?;
    for recipient in prepared.recipients() {
        link.rcpt_to(recipient).await?;
    }
    link.data(&rendered.bytes, caps.size).await?;
    Ok(rendered.bytes)
}
