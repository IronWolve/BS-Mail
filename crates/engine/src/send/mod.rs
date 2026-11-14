//! Sending mail: checking and building the message, and handing it to the
//! account's SMTP server. This file only declares and re-exports the
//! modules.

pub(crate) mod base64;
mod check;
mod date;
mod flowed;
mod header_block;
mod limits;
mod message;
pub(crate) mod random;
mod reply_ids;
mod smtp;
mod transfer;
mod word;

pub use date::now_unix;
pub use message::{Checked, Prepared, Sender, check};
pub use reply_ids::{ReplyIds, parse_threading_headers};
pub use smtp::{SmtpError, SmtpTimeouts, submit};
