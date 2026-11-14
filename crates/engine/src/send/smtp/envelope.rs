//! The steps that carry a message: MAIL FROM, one RCPT TO for each recipient,
//! and DATA. A refusal at any step before DATA means nothing was sent; once
//! DATA has been asked for, only a definite answer settles the matter.

use super::data::dot_stuffed;
use super::error::{SmtpError, Step};
use super::link::Link;

impl Link {
    /// `MAIL FROM`, with the size and 8 bit parameters when they apply.
    pub async fn mail_from(
        &mut self,
        sender: &str,
        size: Option<usize>,
        eight_bit: bool,
        size_limit: Option<u64>,
    ) -> Result<(), SmtpError> {
        let mut line = format!("MAIL FROM:<{sender}>");
        if let Some(size) = size {
            line.push_str(&format!(" SIZE={size}"));
        }
        if eight_bit {
            line.push_str(" BODY=8BITMIME");
        }
        let reply = self.ask(&line, Step::Sender).await?;
        if reply.is_positive() {
            return Ok(());
        }
        if reply.code == 552 {
            return Err(SmtpError::TooBig(size_limit));
        }
        Err(SmtpError::SenderRefused(sender.to_owned(), self.said(&reply)))
    }

    /// `RCPT TO` for one recipient. Any refusal ends the send: a message to
    /// only some of the people it was written for is not what was asked.
    pub async fn rcpt_to(&mut self, recipient: &str) -> Result<(), SmtpError> {
        let reply = self.ask(&format!("RCPT TO:<{recipient}>"), Step::Recipients).await?;
        if reply.is_positive() {
            Ok(())
        } else {
            Err(SmtpError::RecipientRefused(recipient.to_owned(), self.said(&reply)))
        }
    }

    /// `DATA`, the message and the closing dot. Returns once the server has
    /// accepted the message with a 2xx. From the moment DATA is written until
    /// a definite answer, a dropped connection or a silence is reported as
    /// "may have been sent".
    pub async fn data(&mut self, message: &[u8], size_limit: Option<u64>) -> Result<(), SmtpError> {
        self.progress.begin_message();
        let started = self.ask("DATA", Step::StartingMessage).await?;
        if started.code != 354 {
            self.progress.settled();
            return Err(self.refusal_of(&started, size_limit));
        }
        let finish = self.limits.finish;
        self.wire
            .send_raw(&dot_stuffed(message), finish)
            .await
            .map_err(|e| self.lost(e, Step::StartingMessage))?;
        let done = self.wire.read_reply(finish).await.map_err(|e| self.lost(e, Step::StartingMessage))?;
        if done.is_positive() && done.code != 250 {
            // Success is 250. Any other 2xx is not in the standard: whether
            // the message went is not something to guess.
            return Err(SmtpError::MayHaveBeenSent);
        }
        self.progress.settled();
        if done.code == 250 { Ok(()) } else { Err(self.refusal_of(&done, size_limit)) }
    }

    fn refusal_of(&self, reply: &super::reply::Reply, size_limit: Option<u64>) -> SmtpError {
        if reply.code == 552 {
            SmtpError::TooBig(size_limit)
        } else {
            SmtpError::MessageRefused(self.said(reply))
        }
    }
}
