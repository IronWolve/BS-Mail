//! Turning what the person wrote into a message ready for the wire, in two
//! steps so nothing is built before the cheap checks have passed:
//!
//! 1. `check` looks at every address, name, the subject and the size, and
//!    refuses on the first problem. It touches no network and no clock.
//! 2. `Checked::finish` adds what only a message that will really go needs:
//!    the date, a random message id and the reply headers.
//!
//! `Prepared::render` then produces the bytes once the server has said
//! whether it takes 8 bit text.

use std::fmt;

use model::{Address, OutgoingMessage};

use super::check::{Mailbox, Refusal, check_mailbox, check_subject};
use super::date::rfc5322;
use super::flowed::flow;
use super::header_block::{HeaderParts, header_block};
use super::limits::{MAX_MESSAGE_BYTES, MAX_RECIPIENTS};
use super::random::random_hex;
use super::reply_ids::ReplyIds;
use super::transfer::{Encoding, choose, encode};

/// The account the message goes out from.
#[derive(Debug, Clone, Copy)]
pub struct Sender<'a> {
    pub name: &'a str,
    pub address: &'a str,
}

/// A message that passed every check, not yet dated or numbered. Its debug
/// output shows sizes, never what was written.
pub struct Checked {
    from: Mailbox,
    to: Vec<Mailbox>,
    cc: Vec<Mailbox>,
    envelope: Vec<String>,
    subject: String,
    body: String,
}

/// A message with all its headers, waiting for the server's capabilities to
/// settle how the body is encoded. Its debug output shows sizes only.
pub struct Prepared {
    sender: String,
    recipients: Vec<String>,
    headers: String,
    flowed: String,
}

/// The bytes to send, and whether they are 8 bit text. Its debug output shows
/// the size only.
pub struct Rendered {
    pub bytes: Vec<u8>,
    pub eight_bit: bool,
}

impl fmt::Debug for Checked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Checked({} recipients, {} text bytes)", self.envelope.len(), self.body.len())
    }
}

impl fmt::Debug for Prepared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Prepared({} recipients, {} text bytes)", self.recipients.len(), self.flowed.len())
    }
}

impl fmt::Debug for Rendered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rendered({} bytes)", self.bytes.len())
    }
}

/// Checks the message. The first problem found is returned as the reason.
pub fn check(sender: Sender<'_>, message: &OutgoingMessage) -> Result<Checked, Refusal> {
    let from_address = Address { name: Some(sender.name.to_owned()), email: sender.address.to_owned() };
    let from = check_mailbox("the sender (this account's address)", &from_address)?;
    let to = check_all("a recipient in To", &message.to)?;
    let cc = check_all("a recipient in Cc", &message.cc)?;
    let bcc = check_all("a recipient in Bcc", &message.bcc)?;
    let envelope = envelope_of(&[&to, &cc, &bcc])?;
    check_subject(&message.subject)?;
    check_body(&message.body)?;
    Ok(Checked { from, to, cc, envelope, subject: message.subject.clone(), body: message.body.clone() })
}

fn check_all(role: &str, people: &[Address]) -> Result<Vec<Mailbox>, Refusal> {
    people.iter().map(|who| check_mailbox(role, who)).collect()
}

/// Every address once, in the order first seen, hidden ones included. Refuses
/// an empty list and a list over the cap.
fn envelope_of(lists: &[&[Mailbox]]) -> Result<Vec<String>, Refusal> {
    let mut seen: Vec<String> = Vec::new();
    for who in lists.iter().flat_map(|list| list.iter()) {
        let key = dedupe_key(&who.address);
        if !seen.iter().any(|known| dedupe_key(known) == key) {
            seen.push(who.address.clone());
        }
    }
    if seen.is_empty() {
        return Err(Refusal::new("the message has no recipients; nothing was sent"));
    }
    if seen.len() > MAX_RECIPIENTS {
        return Err(Refusal::new(format!(
            "the message has {} recipients; the most this program sends to at once is \
             {MAX_RECIPIENTS}, and nothing was sent",
            seen.len()
        )));
    }
    Ok(seen)
}

/// The domain is not case sensitive; the part before the @ is kept as typed.
fn dedupe_key(address: &str) -> String {
    match address.rsplit_once('@') {
        Some((local, domain)) => format!("{local}@{}", domain.to_ascii_lowercase()),
        None => address.to_owned(),
    }
}

fn check_body(body: &str) -> Result<(), Refusal> {
    if body.len() > MAX_MESSAGE_BYTES {
        return Err(too_big());
    }
    if body.contains('\0') {
        return Err(Refusal::new("the message text contains a NUL character; nothing was sent"));
    }
    Ok(())
}

fn too_big() -> Refusal {
    let megabytes = MAX_MESSAGE_BYTES / (1024 * 1024);
    Refusal::new(format!(
        "the message is larger than {megabytes} MB, the most this program sends; nothing was sent"
    ))
}

impl Checked {
    /// Adds the date (`now`, Unix seconds), a message id that cannot be
    /// guessed and the reply headers, when there are any.
    pub fn finish(self, reply: Option<&ReplyIds>, now: i64) -> Result<Prepared, Refusal> {
        let domain = self.from.address.rsplit_once('@').map_or("localhost", |(_, domain)| domain);
        let random = random_hex::<16>().map_err(Refusal::new)?;
        let headers = header_block(&HeaderParts {
            from: &self.from,
            to: &self.to,
            cc: &self.cc,
            subject: &self.subject,
            date: &rfc5322(now),
            message_id: &format!("{random}@{domain}"),
            reply,
        });
        Ok(Prepared {
            sender: self.from.address,
            recipients: self.envelope,
            headers,
            flowed: flow(&self.body),
        })
    }
}

impl Prepared {
    /// The address for `MAIL FROM`.
    pub fn sender(&self) -> &str {
        &self.sender
    }

    /// The addresses for `RCPT TO`, once each, hidden recipients included.
    pub fn recipients(&self) -> &[String] {
        &self.recipients
    }

    /// The message as it goes on the wire, with CRLF line ends and no dot
    /// stuffing. Refused when it comes to more than the cap.
    pub fn render(&self, eight_bit_ok: bool) -> Result<Rendered, Refusal> {
        let encoding = choose(&self.flowed, eight_bit_ok);
        let body = encode(&self.flowed, encoding);
        let mut bytes = Vec::with_capacity(self.headers.len() + body.len() + 160);
        bytes.extend_from_slice(self.headers.as_bytes());
        bytes.extend_from_slice(b"MIME-Version: 1.0\r\n");
        bytes.extend_from_slice(b"Content-Type: text/plain; charset=utf-8; format=flowed\r\n");
        bytes.extend_from_slice(format!("Content-Transfer-Encoding: {}\r\n\r\n", encoding.name()).as_bytes());
        bytes.extend_from_slice(&body);
        if bytes.len() > MAX_MESSAGE_BYTES {
            return Err(too_big());
        }
        Ok(Rendered { bytes, eight_bit: encoding == Encoding::EightBit })
    }
}

#[cfg(test)]
include!("message.tests.rs");
