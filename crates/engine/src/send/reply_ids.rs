//! The ids that tie a reply to the message it answers. They are read from
//! the original's own headers, which anyone could have written, so each id
//! is checked before it is put into a header of the reply.

/// The longest id kept, in characters.
const MAX_ID_LEN: usize = 200;

/// How many ids a reply's `References` keeps: the first, and the newest.
const MAX_REFERENCES: usize = 25;

/// How much of a message's top is read for its ids. A header is never this
/// long in practice; a hostile one is not read further.
const MAX_HEADER_BYTES: usize = 256 * 1024;

/// What the original message says about itself and its thread. Ids are
/// stored without their angle brackets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplyIds {
    pub message_id: String,
    pub references: Vec<String>,
}

impl ReplyIds {
    /// The same ids with a very long `References` cut down to what a reply
    /// would use anyway: the first id and the newest. What `thread` gives
    /// does not change, but what is kept stays small.
    pub fn compact(mut self) -> Self {
        let keep = MAX_REFERENCES - 1;
        if self.references.len() > keep + 1 {
            let tail = self.references.split_off(self.references.len() - keep);
            self.references.truncate(1);
            self.references.extend(tail);
        }
        self
    }

    /// The value of `In-Reply-To`.
    pub fn in_reply_to(&self) -> String {
        format!("<{}>", self.message_id)
    }

    /// The ids for the `References` of the reply: the original's own
    /// references, then the original itself, each once, never too many.
    pub fn thread(&self) -> Vec<String> {
        let mut ids: Vec<String> = Vec::new();
        for id in self.references.iter().chain(std::iter::once(&self.message_id)) {
            if !ids.contains(id) {
                ids.push(id.clone());
            }
        }
        if ids.len() > MAX_REFERENCES {
            let first = ids[0].clone();
            let tail = ids.split_off(ids.len() - (MAX_REFERENCES - 1));
            ids = std::iter::once(first).chain(tail).collect();
        }
        ids
    }
}

/// Reads `Message-ID` and `References` from header text: the answer to a
/// fetch of just those two fields, or a whole message, of which only the top
/// (up to the first blank line) is looked at. `None` when there is no usable
/// `Message-ID`: a reply then goes out without the thread headers.
pub fn parse_threading_headers(raw: &[u8]) -> Option<ReplyIds> {
    let text = String::from_utf8_lossy(header_part(raw));
    let unfolded = text.replace("\r\n", "\n").replace("\n ", " ").replace("\n\t", " ");
    let mut message_id = None;
    let mut references = Vec::new();
    for line in unfolded.split('\n') {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        match name.trim().to_ascii_lowercase().as_str() {
            "message-id" if message_id.is_none() => {
                message_id = ids_in(value).into_iter().next();
            }
            "references" => references.extend(ids_in(value)),
            _ => {}
        }
    }
    message_id.map(|message_id| ReplyIds { message_id, references }.compact())
}

/// The ids a list row's envelope gives: the message's own `Message-ID`, and
/// its `In-Reply-To` standing in for the `References` an envelope does not
/// carry. `None` when there is no usable `Message-ID`.
pub fn ids_from_envelope(message_id: Option<&[u8]>, in_reply_to: Option<&[u8]>) -> Option<ReplyIds> {
    let own = ids_in(&String::from_utf8_lossy(message_id?)).into_iter().next()?;
    let references = in_reply_to.map(|raw| ids_in(&String::from_utf8_lossy(raw))).unwrap_or_default();
    Some(ReplyIds { message_id: own, references }.compact())
}

/// The top of a message: everything before the first blank line, and never
/// more than `MAX_HEADER_BYTES`.
fn header_part(raw: &[u8]) -> &[u8] {
    let raw = &raw[..raw.len().min(MAX_HEADER_BYTES)];
    let end = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .into_iter()
        .chain(raw.windows(2).position(|w| w == b"\n\n"))
        .min();
    &raw[..end.unwrap_or(raw.len())]
}

/// Every `<id>` in a header value that is safe to repeat in another header.
fn ids_in(value: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = value;
    while let Some(open) = rest.find('<') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('>') else {
            break;
        };
        let id = &after[..close];
        if id_is_safe(id) {
            found.push(id.to_owned());
        }
        rest = &after[close + 1..];
    }
    found
}

fn id_is_safe(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_ID_LEN
        && id.bytes().all(|b| (0x21..=0x7e).contains(&b) && b != b'<' && b != b'>')
}

#[cfg(test)]
include!("reply_ids.tests.rs");
