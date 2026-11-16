//! The safe copy of one message as a whole page: the message's HTML part
//! cleaned by `sanitize`, in a page of this program's own making that also
//! tells the browser, with a content security policy, to load nothing and run
//! nothing even if something got through. The page says in its first line
//! that it is a copy and how many references to the network were left out.

use mail_parser::decoders::html::html_to_text;
use mail_parser::{Message, MessageParser, MimeHeaders, PartType};

use super::dom;
use super::limits::MAX_HTML_BYTES;
use super::sanitize::{Cleaned, clean};
use super::write;
use crate::mime_render::too_nested;
use crate::text_clean::strip_controls;

/// The browser may load pictures that are inside the page (`data:`) and use
/// the page's own style rules, and nothing else: no script, no frame, no
/// font, no request to any host, no `base`, no form.
const POLICY: &str = "default-src 'none'; img-src data:; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'";

/// The page's own look: a quiet note on top, then the message.
const OWN_STYLE: &str = "body{margin:0}.safe-copy-note{background:#eee;color:#222;padding:6px 12px;font:13px sans-serif;\
border-bottom:1px solid #999}.safe-copy-body{padding:8px 12px}";

const CUT_NOTE: &str = "The message is very large, so only its first part is shown.";
const NESTED_NOTE: &str = "The message is formatted in a way that is too complicated to lay out, so only its text is shown.";

/// The safe copy of a message's HTML part.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Page {
    pub html: String,
    /// How many references to the network were left out.
    pub removed: u32,
}

/// What came of asking for the safe copy.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    Page(Page),
    /// The message has no HTML part to open.
    NoHtmlPart,
    /// The bytes are not a message that can be read.
    Unreadable,
}

/// The safe copy of the first HTML part of the message `raw`.
pub(crate) fn build_page(raw: &[u8]) -> Outcome {
    let Some(message) = MessageParser::default().parse(raw) else {
        return Outcome::Unreadable;
    };
    let Some(html) = html_part(&message) else {
        return Outcome::NoHtmlPart;
    };
    let pictures = |id: &str| picture_bytes(&message, id);
    Outcome::Page(page_from_html(html, &pictures))
}

/// The safe copy of `html`, whose `cid:` pictures `pictures` looks up. Markup
/// nested deeper than the text converter takes (the parser's work grows with
/// the square of the depth) is not parsed: its text is shown, and the page
/// says so.
pub(super) fn page_from_html(html: &str, pictures: &dyn Fn(&str) -> Option<Vec<u8>>) -> Page {
    let end = (0..=MAX_HTML_BYTES.min(html.len())).rev().find(|at| html.is_char_boundary(*at)).unwrap_or(0);
    let shown = &html[..end];
    let mut notes = Vec::new();
    if end < html.len() {
        notes.push(CUT_NOTE);
    }
    if too_nested(shown) {
        notes.push(NESTED_NOTE);
        let mut body = String::from("<pre>");
        write::text(&mut body, &html_to_text(shown));
        body.push_str("</pre>");
        let cleaned = Cleaned { body, ..Cleaned::default() };
        return Page { html: assemble(&cleaned, &notes), removed: 0 };
    }
    let cleaned = clean(&dom::parse(shown), pictures);
    Page { html: assemble(&cleaned, &notes), removed: cleaned.removed }
}

fn html_part<'a>(message: &'a Message<'_>) -> Option<&'a str> {
    message.html_body.iter().find_map(|id| match &message.parts.get(*id as usize)?.body {
        PartType::Html(html) if !html.trim().is_empty() => Some(html.as_ref()),
        _ => None,
    })
}

/// The bytes of the part whose Content-ID is `id`, when there is one.
fn picture_bytes(message: &Message<'_>, id: &str) -> Option<Vec<u8>> {
    let wanted = id.trim_matches(|c| c == '<' || c == '>');
    message.parts.iter().find_map(|part| {
        let found = part.content_id()?.trim_matches(|c| c == '<' || c == '>');
        (found == wanted && part.is_binary()).then(|| part.contents().to_vec())
    })
}

fn assemble(cleaned: &Cleaned, notes: &[&str]) -> String {
    let mut note = match cleaned.removed {
        0 => "A safe copy of this message: nothing was left out.".to_owned(),
        1 => "A safe copy of this message: 1 remote item was left out.".to_owned(),
        n => format!("A safe copy of this message: {n} remote items were left out."),
    };
    for more in notes {
        note.push(' ');
        note.push_str(more);
    }
    let mut page = String::with_capacity(cleaned.body.len() + cleaned.styles.len() + 1024);
    page.push_str("<!doctype html><html><head><meta charset=\"utf-8\">");
    page.push_str(&format!("<meta http-equiv=\"Content-Security-Policy\" content=\"{POLICY}\">"));
    page.push_str("<meta name=\"referrer\" content=\"no-referrer\"><title>Message</title>");
    page.push_str(&format!("<style>{OWN_STYLE}</style>"));
    if !cleaned.styles.is_empty() {
        page.push_str(&format!("<style>{}</style>", strip_controls(&cleaned.styles)));
    }
    page.push_str(&format!("</head><body><div class=\"safe-copy-note\">{note}</div><div class=\"safe-copy-body\">"));
    page.push_str(&cleaned.body);
    page.push_str("</div></body></html>");
    page
}

#[cfg(test)]
include!("page.tests.rs");
