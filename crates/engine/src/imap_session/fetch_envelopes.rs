//! Fetching one page of message headers from the open mailbox, newest first.

use async_imap::types::Fetch;
use futures_util::TryStreamExt;
use model::MailDate;

use super::body_structure::has_attachments;
use super::envelope_map::FetchedRow;
use super::error::SessionError;
use super::session::ImapSession;
use super::timeouts::bounded;

/// The data items that fill a list row.
const ROW_ITEMS: &str = "(UID FLAGS INTERNALDATE RFC822.SIZE ENVELOPE BODYSTRUCTURE)";

/// The sequence set for the page that starts `offset` messages from the
/// newest and holds up to `count`, or `None` when the page is empty.
pub fn page_range(exists: u32, offset: u32, count: u32) -> Option<String> {
    if count == 0 || offset >= exists {
        return None;
    }
    let high = exists - offset;
    let low = high.saturating_sub(count - 1).max(1);
    Some(format!("{low}:{high}"))
}

impl ImapSession {
    /// The page of the open mailbox, newest first. `exists` is the mailbox
    /// size from the EXAMINE that opened it.
    pub async fn fetch_page(
        &mut self,
        exists: u32,
        offset: u32,
        count: u32,
    ) -> Result<Vec<Fetch>, SessionError> {
        let Some(range) = page_range(exists, offset, count) else {
            return Ok(Vec::new());
        };
        let limit = self.limits.command;
        let session = &mut self.inner;
        let result = bounded(limit, async {
            session.fetch(range, ROW_ITEMS).await?.try_collect().await
        })
        .await;
        let mut rows: Vec<Fetch> = self.settle(result)?;
        rows.sort_by_key(|row| std::cmp::Reverse(row.message));
        Ok(rows)
    }
}

/// The parts of a FETCH response a list row needs; `None` when the server
/// left out the UID.
pub fn fetched_row(fetch: &Fetch) -> Option<FetchedRow<'_>> {
    let uid = fetch.uid?;
    Some(FetchedRow {
        uid,
        flags: fetch.flags().collect(),
        envelope: fetch.envelope(),
        internal_date: fetch.internal_date().map(|date| MailDate {
            unix: date.timestamp(),
            offset_minutes: i16::try_from(date.offset().local_minus_utc() / 60).unwrap_or(0),
        }),
        size: fetch.size.unwrap_or(0),
        has_attachments: fetch.bodystructure().is_some_and(has_attachments),
    })
}

#[cfg(test)]
include!("fetch_envelopes.tests.rs");
