//! From the loaded rows to the rows the list draws.
//!
//! The loaded rows stay in the order the engine gave them (newest arrival
//! first), which is what pages merge into. The rows drawn are made from them
//! each time anything changes: grouped into threads when those are on, sorted
//! when a sort is chosen, with the children of the open threads below their
//! thread's row. Sorting is stable: rows that tie keep the order they were
//! loaded in.

use model::{Envelope, MessageId};

use super::arrangement::Arrangement;
use super::row_meta::{RowKind, RowMeta, ThreadInfo};
use super::sort::{Sort, SortKey, key_of};
use super::threads::group;

/// The rows made from the loaded ones.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Arranged {
    /// The rows, in the order they are drawn.
    pub envelopes: Vec<Envelope>,
    /// What each row is, one for each row; empty when every row is a message
    /// on its own.
    pub rows: Vec<RowMeta>,
    pub threads: Vec<ThreadInfo>,
}

/// A row before the open threads are spread out: one message, or a thread.
struct Unit {
    /// Loaded indexes, oldest first; one for a message on its own.
    members: Vec<usize>,
    /// The loaded index of the newest member: the one the row shows.
    head: usize,
    /// The position of its first member among the loaded rows, which is what
    /// ties are broken by.
    rank: usize,
}

/// Arranges `loaded` for `sort`, with or without threads. `recipients` tells
/// whether the sender column names the recipient for a message.
pub fn arrange(
    loaded: &[Envelope],
    sort: Option<Sort>,
    arrangement: &Arrangement,
    recipients: &dyn Fn(&Envelope) -> bool,
) -> Arranged {
    let units = if arrangement.threaded { thread_units(loaded) } else { single_units(loaded) };
    let shown: Vec<Envelope> = units.iter().map(|unit| shown_row(loaded, unit)).collect();
    let order = ordered(&units, &shown, sort, recipients);
    let mut rows: Vec<Option<Envelope>> = shown.into_iter().map(Some).collect();
    let mut out = Arranged::default();
    for at in order {
        let unit = &units[at];
        let row = rows[at].take().unwrap_or_else(|| loaded[unit.head].clone());
        push_unit(&mut out, loaded, unit, row, arrangement);
    }
    if out.rows.iter().all(|row| row.kind == RowKind::Single) {
        out.rows.clear();
    }
    out
}

fn single_units(loaded: &[Envelope]) -> Vec<Unit> {
    (0..loaded.len()).map(|at| Unit { members: vec![at], head: at, rank: at }).collect()
}

fn thread_units(loaded: &[Envelope]) -> Vec<Unit> {
    group(loaded)
        .into_iter()
        .map(|members| {
            let head = newest(loaded, &members);
            let rank = members.iter().copied().min().unwrap_or(head);
            Unit { members, head, rank }
        })
        .collect()
}

/// The newest of `members` by date; of equal dates the one loaded first.
fn newest(loaded: &[Envelope], members: &[usize]) -> usize {
    let mut best = members[0];
    for &at in &members[1..] {
        let (date, best_date) = (loaded[at].date.unix, loaded[best].date.unix);
        if date > best_date || (date == best_date && at < best) {
            best = at;
        }
    }
    best
}

/// The row that stands for a unit: the message itself, or for a thread its
/// newest message with the marks, size and attachments of all of them.
fn shown_row(loaded: &[Envelope], unit: &Unit) -> Envelope {
    let mut row = loaded[unit.head].clone();
    if unit.members.len() < 2 {
        return row;
    }
    let all = || unit.members.iter().map(|&at| &loaded[at]);
    row.flags.seen = all().all(|envelope| envelope.flags.seen);
    row.flags.flagged = all().any(|envelope| envelope.flags.flagged);
    row.flags.alert = all().any(|envelope| envelope.flags.alert);
    row.has_attachments = all().any(|envelope| envelope.has_attachments);
    row.size = all().fold(0u64, |sum, envelope| sum.saturating_add(envelope.size));
    row
}

/// The units' places in the order they are drawn.
fn ordered(units: &[Unit], shown: &[Envelope], sort: Option<Sort>, recipients: &dyn Fn(&Envelope) -> bool) -> Vec<usize> {
    let mut order: Vec<usize> = (0..units.len()).collect();
    let Some(sort) = sort else {
        order.sort_by_key(|&at| units[at].rank);
        return order;
    };
    let keys: Vec<SortKey> = shown.iter().map(|row| key_of(sort.column, row, recipients(row))).collect();
    order.sort_by(|&a, &b| {
        let by_key = keys[a].cmp(&keys[b]);
        let by_key = if sort.descending { by_key.reverse() } else { by_key };
        by_key.then_with(|| units[a].rank.cmp(&units[b].rank))
    });
    order
}

/// Adds a unit's row, and for an open thread the rest of its messages.
fn push_unit(out: &mut Arranged, loaded: &[Envelope], unit: &Unit, row: Envelope, arrangement: &Arrangement) {
    if unit.members.len() < 2 {
        out.envelopes.push(row);
        out.rows.push(RowMeta::SINGLE);
        return;
    }
    let ids: Vec<MessageId> = unit.members.iter().map(|&at| loaded[at].id).collect();
    let open = arrangement.is_open(&ids);
    let thread = out.threads.len();
    let count = ids.len();
    out.threads.push(ThreadInfo { members: ids, head: loaded[unit.head].id });
    out.envelopes.push(row);
    out.rows.push(RowMeta { kind: RowKind::Head { open }, count, thread });
    if open {
        for &at in unit.members.iter().filter(|&&at| at != unit.head) {
            out.envelopes.push(loaded[at].clone());
            out.rows.push(RowMeta { kind: RowKind::Child, count, thread });
        }
    }
}

#[cfg(test)]
include!("arrange.tests.rs");
