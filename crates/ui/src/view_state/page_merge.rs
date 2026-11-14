//! Putting a page of envelopes together with the ones already shown, so that
//! older mail that was loaded stays when the newest page is read again, and a
//! page of older mail joins the end of the list.

use std::collections::HashSet;

use model::{Envelope, MessageId};

/// A page of older mail joins the end of the list: the messages not shown yet
/// go after the ones shown.
pub fn append_older(shown: Vec<Envelope>, older: Vec<Envelope>) -> Vec<Envelope> {
    let mut merged = shown;
    let known: HashSet<MessageId> = merged.iter().map(|envelope| envelope.id).collect();
    merged.extend(older.into_iter().filter(|envelope| !known.contains(&envelope.id)));
    merged
}

/// The list after the newest page was read again. `old` is what was shown and
/// `total` the messages in the folder.
///
/// The new page replaces the top of the list and the loaded mail that is
/// older than it is kept, but only when the new page does not hold the whole
/// folder and overlaps what was shown: that is how it is known to be the same
/// folder with the same numbering, and a folder rebuilt on the server never
/// keeps old rows.
pub fn merge_newest(old: Vec<Envelope>, total: u32, new: Vec<Envelope>) -> Vec<Envelope> {
    if usize::try_from(total).is_ok_and(|all| all <= new.len()) {
        return new;
    }
    let Some(oldest) = new.last().map(|envelope| envelope.uid) else {
        return new;
    };
    let fresh: HashSet<MessageId> = new.iter().map(|envelope| envelope.id).collect();
    if !old.iter().any(|envelope| fresh.contains(&envelope.id)) {
        return new;
    }
    let older: Vec<Envelope> =
        old.into_iter().filter(|envelope| envelope.uid < oldest && !fresh.contains(&envelope.id)).collect();
    new.into_iter().chain(older).collect()
}

#[cfg(test)]
include!("page_merge.tests.rs");
