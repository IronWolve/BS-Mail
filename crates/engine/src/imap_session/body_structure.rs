//! Whether a message carries attachments, judged from its BODYSTRUCTURE
//! alone, so the list can show the paperclip without downloading the message.

use async_imap::imap_proto::{BodyContentCommon, BodyStructure};

/// Content types that ride along with signed mail and are not files the
/// reader would want to open.
const SIGNATURE_TYPES: [(&str, &str); 3] = [
    ("application", "pgp-signature"),
    ("application", "pkcs7-signature"),
    ("application", "x-pkcs7-signature"),
];

/// True when any part is a file the reader can save: marked as an
/// attachment, a forwarded message, or a non-text part that is not an inline
/// picture of the HTML body.
pub fn has_attachments(structure: &BodyStructure<'_>) -> bool {
    match structure {
        BodyStructure::Multipart { bodies, .. } => bodies.iter().any(has_attachments),
        BodyStructure::Message { .. } => true,
        BodyStructure::Text { common, .. } => disposition_is(common, "attachment"),
        BodyStructure::Basic { common, other, .. } => {
            if is_signature(common) {
                return false;
            }
            let inline_picture = disposition_is(common, "inline") && other.id.is_some();
            !inline_picture
        }
    }
}

fn disposition_is(common: &BodyContentCommon<'_>, kind: &str) -> bool {
    common
        .disposition
        .as_ref()
        .is_some_and(|d| d.ty.eq_ignore_ascii_case(kind))
}

fn is_signature(common: &BodyContentCommon<'_>) -> bool {
    SIGNATURE_TYPES.iter().any(|(ty, subtype)| {
        common.ty.ty.eq_ignore_ascii_case(ty) && common.ty.subtype.eq_ignore_ascii_case(subtype)
    })
}

#[cfg(test)]
include!("body_structure.tests.rs");
