//! A raw message to the body the reading pane shows.

use mail_parser::MessageParser;
use model::Body;

use super::attachments::list_attachments;
use super::body_select::pick_body;
use super::reply_to::reply_to_of;
use super::styled_lines::style_lines;

/// Parses `raw` and builds the body: the text as styled lines and the list
/// of attachments. Bytes that do not parse as a message are shown as text.
pub fn render_body(raw: &[u8]) -> Body {
    let Some(message) = MessageParser::default().parse(raw) else {
        return Body {
            lines: style_lines(&String::from_utf8_lossy(raw)),
            attachments: Vec::new(),
            reply_to: Vec::new(),
        };
    };
    let text = pick_body(&message).unwrap_or_default();
    Body {
        lines: style_lines(&text),
        attachments: list_attachments(&message),
        reply_to: reply_to_of(&message),
    }
}

#[cfg(test)]
include!("render.tests.rs");
