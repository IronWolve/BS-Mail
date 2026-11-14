//! The caps on one outgoing message, gathered so every check and every
//! message about them says the same thing.

/// The most a whole message may weigh once built, in bytes (25 MB).
pub const MAX_MESSAGE_BYTES: usize = 25 * 1024 * 1024;

/// The most recipients (To, Cc and Bcc together, each address once).
pub const MAX_RECIPIENTS: usize = 100;

/// The longest subject, in characters.
pub const MAX_SUBJECT_CHARS: usize = 900;

/// The longest display name, in characters.
pub const MAX_NAME_CHARS: usize = 200;

/// The longest mail address, in bytes (RFC 5321 path limit).
pub const MAX_ADDRESS_BYTES: usize = 254;
