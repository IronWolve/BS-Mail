//! Which attachments are asked about before the system opens them. Programs,
//! scripts and pages that run code are never opened at all (the program side
//! refuses them, see `attachment_name`); these are the formats that are opened
//! but can carry active content of their own: PDF, the older Office documents
//! (word processor, spreadsheet and presentation, with their templates and
//! show files) and RTF. Newer Office files without macros, pictures, text and
//! archives open without a question.

/// Last extensions (lower case) of files that ask first.
const ASKED_EXTENSIONS: &[&str] = &["pdf", "doc", "dot", "xls", "xlt", "xla", "ppt", "pps", "pot", "ppa", "rtf"];

/// Content types that ask first whatever the file is called.
const ASKED_TYPES: &[&str] = &[
    "application/pdf",
    "application/x-pdf",
    "application/msword",
    "application/vnd.ms-excel",
    "application/vnd.ms-powerpoint",
    "application/rtf",
    "text/rtf",
];

/// Whether opening the attachment `name` of `content_type` asks first. The
/// name is judged by its last extension only (`report.pdf.txt` is text), and a
/// name with no dot (`pdf`) has no extension.
pub fn may_carry_active_content(name: &str, content_type: &str) -> bool {
    let last = name.rsplit(['/', '\\']).next().unwrap_or("").trim();
    let by_name = last.rsplit_once('.').is_some_and(|(_, extension)| ASKED_EXTENSIONS.contains(&extension.trim().to_ascii_lowercase().as_str()));
    let bare = content_type.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
    by_name || ASKED_TYPES.contains(&bare.as_str())
}

#[cfg(test)]
include!("active_content.tests.rs");
