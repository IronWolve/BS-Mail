//! The one table that pairs a file extension with a content type. A file to
//! attach gets its type from its extension (never from what is inside it),
//! and a received attachment that came with no name gets an extension from
//! its type. A name or type that is not in the table has no match.

/// (extension, content type): lowercase, no dot. The first row for a type is
/// the extension chosen for a nameless attachment of that type.
const TABLE: &[(&str, &str)] = &[
    ("pdf", "application/pdf"),
    ("txt", "text/plain"),
    ("log", "text/plain"),
    ("md", "text/markdown"),
    ("csv", "text/csv"),
    ("html", "text/html"),
    ("htm", "text/html"),
    ("css", "text/css"),
    ("ics", "text/calendar"),
    ("vcf", "text/vcard"),
    ("xml", "application/xml"),
    ("json", "application/json"),
    ("zip", "application/zip"),
    ("gz", "application/gzip"),
    ("tar", "application/x-tar"),
    ("7z", "application/x-7z-compressed"),
    ("rar", "application/vnd.rar"),
    ("doc", "application/msword"),
    ("docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
    ("xls", "application/vnd.ms-excel"),
    ("xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
    ("ppt", "application/vnd.ms-powerpoint"),
    ("pptx", "application/vnd.openxmlformats-officedocument.presentationml.presentation"),
    ("odt", "application/vnd.oasis.opendocument.text"),
    ("ods", "application/vnd.oasis.opendocument.spreadsheet"),
    ("odp", "application/vnd.oasis.opendocument.presentation"),
    ("rtf", "application/rtf"),
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("bmp", "image/bmp"),
    ("svg", "image/svg+xml"),
    ("tif", "image/tiff"),
    ("tiff", "image/tiff"),
    ("ico", "image/x-icon"),
    ("mp3", "audio/mpeg"),
    ("wav", "audio/wav"),
    ("ogg", "audio/ogg"),
    ("flac", "audio/flac"),
    ("m4a", "audio/mp4"),
    ("mp4", "video/mp4"),
    ("mov", "video/quicktime"),
    ("webm", "video/webm"),
    ("mkv", "video/x-matroska"),
    ("avi", "video/x-msvideo"),
    ("eml", "message/rfc822"),
];

/// The type a file name's extension stands for, or `None` for an extension
/// that is not in the table (the caller then says `application/octet-stream`).
pub fn type_for_name(name: &str) -> Option<&'static str> {
    let (_, extension) = name.rsplit_once('.')?;
    let lower = extension.to_ascii_lowercase();
    TABLE.iter().find(|(known, _)| *known == lower).map(|(_, ty)| *ty)
}

/// The extension (no dot) to give a file of this content type, or `None`
/// when the type is not in the table. Parameters after a `;` are ignored.
pub fn extension_for_type(content_type: &str) -> Option<&'static str> {
    let bare = content_type.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
    TABLE.iter().find(|(_, ty)| *ty == bare).map(|(extension, _)| *extension)
}

#[cfg(test)]
include!("media_types.tests.rs");
