//! Sending mail: checking and building the message, and handing it to the
//! account's SMTP server. This file only declares and re-exports the
//! modules.

pub(crate) mod base64;
mod attachment_header;
mod check;
mod check_files;
mod date;
mod flowed;
mod header_block;
mod idn;
mod limits;
mod message;
mod multipart;
pub(crate) mod random;
mod reply_ids;
mod smtp;
mod transfer;
mod word;

#[cfg(test)]
pub(crate) use transfer::quoted_printable as quoted_printable_for_tests;

pub use date::now_unix;
pub use message::{Checked, Prepared, Sender, check};
pub use reply_ids::{ReplyIds, ids_from_envelope, parse_threading_headers};
pub use smtp::{SmtpError, SmtpTimeouts, submit};
