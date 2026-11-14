//! How much of a message the reading pane works through. A message can be
//! made to be as large or as deeply nested as its sender likes, so each step
//! has a ceiling, and what is left out is said on screen.

use std::time::Duration;

/// The most text shown from one message.
pub(super) const MAX_TEXT_BYTES: usize = 4 * 1024 * 1024;
/// The most HTML handed to the converter.
pub(super) const MAX_HTML_BYTES: usize = 2 * 1024 * 1024;
/// Nested tags beyond this are not handed to the converter, which works down
/// the nesting one level at a time.
pub(super) const MAX_HTML_DEPTH: usize = 400;
/// The most attachments listed.
pub(super) const MAX_ATTACHMENTS: usize = 200;
/// How long one message may take to turn into text before it is given up on.
pub(super) const RENDER_LIMIT: Duration = Duration::from_secs(5);
/// The room the reading thread has to work down nested parts.
pub(super) const RENDER_STACK: usize = 64 * 1024 * 1024;
/// How many given-up reading threads may still be running before no new one
/// is started: they cannot be stopped, only waited out.
pub(super) const MAX_RUNAWAY: usize = 2;

/// How many bytes of `text` to keep so at most `max` remain, never cutting a
/// character in two.
pub(super) fn keep(text: &str, max: usize) -> usize {
    if text.len() <= max {
        return text.len();
    }
    (0..=max).rev().find(|&at| text.is_char_boundary(at)).unwrap_or(0)
}

/// Megabytes, rounded up, for the notices.
pub(super) fn megabytes(bytes: u64) -> u64 {
    bytes.div_ceil(1024 * 1024)
}

#[cfg(test)]
include!("limits.tests.rs");
