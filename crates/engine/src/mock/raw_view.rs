//! A stand-in for the raw text of a message, for the mock backend: it holds
//! no source, so the headers are written from the envelope and the source is
//! those headers, a blank line and the body's lines.

use model::{Address, Envelope, Event, MessageId, RawPart};

use super::Fixture;

fn written(addresses: &[Address]) -> String {
    addresses
        .iter()
        .map(|address| match &address.name {
            Some(name) => format!("{name} <{}>", address.email),
            None => address.email.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn header_block(envelope: &Envelope) -> String {
    let mut text = format!("From: {}\n", written(&envelope.from));
    if !envelope.to.is_empty() {
        text.push_str(&format!("To: {}\n", written(&envelope.to)));
    }
    if !envelope.cc.is_empty() {
        text.push_str(&format!("Cc: {}\n", written(&envelope.cc)));
    }
    text.push_str(&format!("Subject: {}\n", envelope.subject));
    if let Some(id) = &envelope.message_id {
        text.push_str(&format!("Message-ID: {id}\n"));
    }
    text
}

/// The raw text event for `message`, or `None` when the fixture has no such message.
pub(crate) fn raw_event(data: &Fixture, message: MessageId, part: RawPart) -> Option<Event> {
    let envelope = data.messages.iter().find(|envelope| envelope.id == message)?;
    let mut text = header_block(envelope);
    if part == RawPart::Source {
        text.push('\n');
        if message == data.open_message {
            for line in &data.open_body.lines {
                text.push_str(&line.spans.iter().map(|span| span.text.as_str()).collect::<String>());
                text.push('\n');
            }
        }
    }
    Some(Event::RawText { message, part, text })
}

#[cfg(test)]
include!("raw_view.tests.rs");
