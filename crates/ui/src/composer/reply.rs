//! Starting a message from another one: a reply, a reply to everyone, or a
//! forward.

use model::{Address, Body, Envelope};

/// What a new message starts with.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Draft {
    pub to: Vec<Address>,
    pub cc: Vec<Address>,
    pub subject: String,
    pub body: String,
    reply: bool,
}

impl Draft {
    /// A reply starts with the cursor in the body, above the quote; a draft
    /// with nobody to send to yet starts at the recipient.
    pub fn is_reply(&self) -> bool {
        self.reply
    }
}

/// The message being answered or forwarded, in the forms a draft needs.
pub struct Original<'a> {
    pub envelope: &'a Envelope,
    /// Its body as plain text.
    pub text: String,
    /// Its date as the reader shows it.
    pub date: String,
}

/// A reply to the sender; with `everyone`, also to the others on the message
/// except the user's own address.
pub fn reply(original: &Original<'_>, own: &str, everyone: bool) -> Draft {
    let envelope = original.envelope;
    let to: Vec<Address> = envelope.from.iter().filter(|a| !is_same(&a.email, own)).cloned().collect();
    let to = if to.is_empty() { envelope.from.clone() } else { to };
    let mut cc = Vec::new();
    if everyone {
        for address in envelope.to.iter().chain(&envelope.cc) {
            let known = is_same(&address.email, own)
                || to.iter().chain(&cc).any(|other: &Address| is_same(&other.email, &address.email));
            if !known {
                cc.push(address.clone());
            }
        }
    }
    let attribution = format!("On {}, {} wrote:", original.date, sender(envelope));
    Draft {
        to,
        cc,
        subject: prefixed("Re:", &envelope.subject),
        body: format!("\n\n{attribution}\n{}\n", quote(&original.text)),
        reply: true,
    }
}

/// A forward: nobody to send to yet, the original after a marker line.
pub fn forward(original: &Original<'_>) -> Draft {
    let envelope = original.envelope;
    let recipients = |list: &[Address]| super::address_list::format(list);
    let mut head = format!(
        "\n\n---------- Forwarded message ----------\nFrom: {}\nDate: {}\nSubject: {}\n",
        recipients(&envelope.from),
        original.date,
        envelope.subject
    );
    if !envelope.to.is_empty() {
        head.push_str(&format!("To: {}\n", recipients(&envelope.to)));
    }
    Draft {
        subject: prefixed("Fwd:", &envelope.subject),
        body: format!("{head}\n{}\n", original.text),
        ..Draft::default()
    }
}

/// The body as plain text, one line per styled line.
pub fn body_text(body: &Body) -> String {
    body.lines
        .iter()
        .map(|line| line.spans.iter().map(|span| span.text.as_str()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

/// `Re: ` or `Fwd: ` in front of a subject, once.
fn prefixed(prefix: &str, subject: &str) -> String {
    let subject = subject.trim();
    let bare = prefix.trim_end_matches(':');
    let already = subject
        .get(..prefix.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(prefix))
        || (bare.eq_ignore_ascii_case("Fwd")
            && subject.get(..3).is_some_and(|start| start.eq_ignore_ascii_case("fw:")));
    if already { subject.to_string() } else { format!("{prefix} {subject}") }
}

fn quote(text: &str) -> String {
    text.lines()
        .map(|line| if line.is_empty() { ">".to_string() } else { format!("> {line}") })
        .collect::<Vec<_>>()
        .join("\n")
}

fn sender(envelope: &Envelope) -> String {
    envelope.from.first().map_or_else(String::new, |address| {
        address.name.clone().filter(|name| !name.is_empty()).unwrap_or_else(|| address.email.clone())
    })
}

fn is_same(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

#[cfg(test)]
include!("reply.tests.rs");
