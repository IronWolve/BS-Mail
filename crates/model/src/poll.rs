//! How often a mail account is asked whether the open folder changed. The
//! engine enforces these when it reads an account file and the account
//! screens show them, so they are written once, here.

/// How often the program checks for new mail when the file says nothing.
pub const DEFAULT_POLL_SECONDS: u32 = 60;

/// The shortest wait between two checks; a smaller non-zero value is raised.
/// Zero is allowed and means never.
pub const MIN_POLL_SECONDS: u32 = 15;
