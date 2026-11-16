//! Fetching one attachment: the part is found in the server's own account of
//! the message (BODYSTRUCTURE), which says how it is encoded and how large it
//! is before any of it is downloaded, and then read with `BODY.PEEK[n]` in
//! slices of `SLICE_BYTES`. Each slice is decoded as it arrives (see
//! `part_decode`), so what is held at any moment is the file so far and one
//! slice, never the whole encoded part as well. PEEK leaves the message
//! unread. The part is refused, before a byte of it is fetched, when the
//! server says it is larger than the ceiling, and again if the slices come to
//! more than the ceiling anyway. The server's size is never used to stop a
//! download early: only a short slice ends the part.

use std::time::Instant;

use async_imap::imap_proto::{BodyStructure, SectionPath};
use futures_util::TryStreamExt;

use super::error::SessionError;
use super::part_decode::{PartDecoder, Transfer};
use super::session::ImapSession;
use super::timeouts::{BODY_CEILING, bounded, bounded_by_progress};

/// How much of a part is asked for at a time.
pub const SLICE_BYTES: u32 = 1024 * 1024;

/// What the server says about one part of a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartFacts {
    pub transfer: Transfer,
    /// The part's size as it is sent, in bytes.
    pub wire_size: u64,
    /// `type/subtype`, lower case.
    pub content_type: String,
}

/// How a fetch of one part ended.
#[derive(Debug, PartialEq, Eq)]
pub enum Fetched {
    /// The file, decoded.
    File(Vec<u8>),
    /// More than the ceiling came: the file is not kept.
    TooBig,
    /// The download took longer than a message is given in all.
    TooSlow,
}

/// The part numbered `path` ("2", "1.3" as [1, 3]) in `structure`.
pub fn find_part(structure: &BodyStructure<'_>, path: &[u32]) -> Option<PartFacts> {
    let mut at = structure;
    if !matches!(structure, BodyStructure::Multipart { .. }) {
        // A message that is not multipart is the one part "1".
        return (path == [1]).then(|| facts_of(structure)).flatten();
    }
    for number in path {
        let BodyStructure::Multipart { bodies, .. } = at else {
            return None;
        };
        at = bodies.get(usize::try_from(number.checked_sub(1)?).ok()?)?;
    }
    facts_of(at)
}

fn facts_of(part: &BodyStructure<'_>) -> Option<PartFacts> {
    let (common, other) = match part {
        BodyStructure::Basic { common, other, .. }
        | BodyStructure::Text { common, other, .. }
        | BodyStructure::Message { common, other, .. } => (common, other),
        BodyStructure::Multipart { .. } => return None,
    };
    Some(PartFacts {
        transfer: Transfer::of(&other.transfer_encoding),
        wire_size: u64::from(other.octets),
        content_type: format!("{}/{}", common.ty.ty, common.ty.subtype).to_ascii_lowercase(),
    })
}

impl ImapSession {
    /// What the server says about the part `path` of the message with this
    /// UID in the open mailbox, or `None` when the message or the part is not
    /// there.
    pub async fn part_facts(&mut self, uid: u32, path: &[u32]) -> Result<Option<PartFacts>, SessionError> {
        let limit = self.limits.command;
        let session = &mut self.inner;
        let result = bounded(limit, async {
            session.uid_fetch(uid.to_string(), "(UID BODYSTRUCTURE)").await?.try_collect().await
        })
        .await;
        let rows: Vec<_> = self.settle(result)?;
        Ok(rows
            .iter()
            .filter(|row| row.uid == Some(uid))
            .find_map(|row| row.bodystructure().and_then(|structure| find_part(structure, path))))
    }

    /// The part `path` of the message, decoded, read in slices. `facts` is
    /// what `part_facts` said; no more than `ceiling` bytes of it are taken.
    pub async fn fetch_part(
        &mut self,
        uid: u32,
        path: &[u32],
        facts: &PartFacts,
        ceiling: u64,
    ) -> Result<Fetched, SessionError> {
        let started = Instant::now();
        let mut decoder = PartDecoder::new(facts.transfer);
        let mut offset: u64 = 0;
        loop {
            if started.elapsed() >= BODY_CEILING {
                return Ok(Fetched::TooSlow);
            }
            let slice = self.fetch_slice(uid, path, offset, started).await?;
            offset += slice.len() as u64;
            if offset > ceiling {
                return Ok(Fetched::TooBig);
            }
            // Only a short slice ends the part: the size the server reported is
            // trusted to refuse a download, never to cut one short.
            let last = slice.len() < SLICE_BYTES as usize;
            decoder.push(&slice);
            if last {
                return Ok(Fetched::File(decoder.finish()));
            }
        }
    }

    /// `count` bytes from `offset` of the part, fewer at its end.
    async fn fetch_slice(&mut self, uid: u32, path: &[u32], offset: u64, started: Instant) -> Result<Vec<u8>, SessionError> {
        let silence = self.limits.body;
        let arrived = self.inner.get_ref().read_counter();
        let session = &mut self.inner;
        let number = path.iter().map(u32::to_string).collect::<Vec<_>>().join(".");
        let query = format!("(UID BODY.PEEK[{number}]<{offset}.{SLICE_BYTES}>)");
        let left = BODY_CEILING.saturating_sub(started.elapsed());
        let result = bounded_by_progress(silence, left, &arrived, async {
            session.uid_fetch(uid.to_string(), &query).await?.try_collect().await
        })
        .await;
        let rows: Vec<_> = self.settle(result)?;
        let wanted = SectionPath::Part(path.to_vec(), None);
        Ok(rows
            .iter()
            .filter(|row| row.uid == Some(uid))
            .find_map(|row| row.section(&wanted))
            .map(<[u8]>::to_vec)
            .unwrap_or_default())
    }
}

#[cfg(test)]
include!("fetch_part.tests.rs");
