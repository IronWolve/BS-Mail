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
    merge_newest_without(old, total, new, &[])
}

/// As `merge_newest`, for when some messages were asked to be moved or
/// deleted (`gone`). A loaded row below the new page cannot be told gone by
/// looking at the page, but the folder's count can: the folder holds
/// `total` less the page's rows below the page, so loaded rows beyond that
/// are gone. The ones named in `gone` are taken out (no more than the count
/// says, since the change may not have happened yet); when rows are gone
/// that nobody named, which one is not known, so no row below the page is
/// kept and the older mail is read again when it is scrolled to.
pub fn merge_newest_without(
    old: Vec<Envelope>,
    total: u32,
    new: Vec<Envelope>,
    gone: &[MessageId],
) -> Vec<Envelope> {
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
    let mut older: Vec<Envelope> =
        old.into_iter().filter(|envelope| envelope.uid < oldest && !fresh.contains(&envelope.id)).collect();
    let below = usize::try_from(total).unwrap_or(usize::MAX).saturating_sub(new.len());
    let mut excess = older.len().saturating_sub(below);
    if excess > 0 {
        older.retain(|envelope| {
            let named = excess > 0 && gone.contains(&envelope.id);
            if named {
                excess -= 1;
            }
            !named
        });
    }
    if excess > 0 {
        return new;
    }
    new.into_iter().chain(older).collect()
}

#[cfg(test)]
include!("page_merge.tests.rs");
