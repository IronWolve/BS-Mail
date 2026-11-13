//! The list of non-body parts under the header block.

use mail_parser::{Message, MessagePart, MimeHeaders, PartType};
use model::AttachmentInfo;

use crate::text_clean::single_line;

/// Every attachment: the part's index in the message's part tree, its file
/// name, its type and its decoded size.
pub fn list_attachments(message: &Message<'_>) -> Vec<AttachmentInfo> {
    message
        .attachments
        .iter()
        .enumerate()
        .filter_map(|(position, &id)| {
            let part = message.parts.get(id as usize)?;
            Some(AttachmentInfo {
                part: id,
                name: name_of(part, position),
                content_type: type_of(part),
                size: part.len() as u64,
            })
        })
        .collect()
}

fn name_of(part: &MessagePart<'_>, position: usize) -> String {
    let named = part
        .attachment_name()
        .map(single_line)
        .map(|n| n.trim().to_owned())
        .filter(|n| !n.is_empty());
    if let Some(name) = named {
        return name;
    }
    if let PartType::Message(inner) = &part.body {
        let subject = inner.subject().map(single_line).unwrap_or_default();
        let subject = subject.trim();
        return if subject.is_empty() {
            "message.eml".to_owned()
        } else {
            format!("{subject}.eml")
        };
    }
    format!("attachment-{}", position + 1)
}

fn type_of(part: &MessagePart<'_>) -> String {
    match part.content_type() {
        Some(found) => {
            let subtype = found.subtype().unwrap_or("octet-stream");
            format!("{}/{}", found.ctype(), subtype).to_ascii_lowercase()
        }
        None => match part.body {
            PartType::Message(_) => "message/rfc822",
            PartType::Text(_) => "text/plain",
            PartType::Html(_) => "text/html",
            _ => "application/octet-stream",
        }
        .to_owned(),
    }
}

#[cfg(test)]
include!("attachments.tests.rs");
