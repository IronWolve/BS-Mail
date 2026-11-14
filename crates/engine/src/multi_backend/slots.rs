//! Id namespaces: how one number carries both the member that owns an item
//! and the member's own database id.
//!
//! Every member numbers its folders, messages and threads from 1 by itself, so
//! two members would hand the screen the same id for different things. The
//! composite gives each member a slot number, and the id the screen sees is
//! `slot * 2^40 + local`. Slot 0 leaves ids unchanged, so a program with one
//! account sees exactly what the member sends.

use model::{FolderId, MessageId, ThreadId};

/// A member's place in the composite's numbering; never reused.
pub type Slot = u64;

/// How many low bits of a global id hold the member's own id.
pub const SLOT_BITS: u32 = 40;

/// A member's own ids must be below this (and not negative).
pub const LOCAL_LIMIT: i64 = 1 << SLOT_BITS;

/// The highest slot whose ids still fit in an `i64`: `2^23 - 1`.
pub const MAX_SLOT: Slot = 0x7F_FFFF;

/// The global id for a member's own id, or `None` when either part does not
/// fit: a negative local id, one of `2^40` or more, or a slot above
/// [`MAX_SLOT`].
pub fn encode_id(slot: Slot, local: i64) -> Option<i64> {
    if !(0..LOCAL_LIMIT).contains(&local) || slot > MAX_SLOT {
        return None;
    }
    i64::try_from(slot).ok()?.checked_mul(LOCAL_LIMIT)?.checked_add(local)
}

/// Splits a global id into its slot and the member's own id, or `None` for a
/// negative id, which this numbering never produces.
pub fn decode_id(global: i64) -> Option<(Slot, i64)> {
    if global < 0 {
        return None;
    }
    let slot = Slot::try_from(global >> SLOT_BITS).ok()?;
    Some((slot, global & (LOCAL_LIMIT - 1)))
}

pub fn encode_folder(slot: Slot, local: FolderId) -> Option<FolderId> {
    encode_id(slot, local.0).map(FolderId)
}

pub fn encode_message(slot: Slot, local: MessageId) -> Option<MessageId> {
    encode_id(slot, local.0).map(MessageId)
}

pub fn encode_thread(slot: Slot, local: ThreadId) -> Option<ThreadId> {
    encode_id(slot, local.0).map(ThreadId)
}

pub fn decode_folder(global: FolderId) -> Option<(Slot, FolderId)> {
    decode_id(global.0).map(|(slot, local)| (slot, FolderId(local)))
}

pub fn decode_message(global: MessageId) -> Option<(Slot, MessageId)> {
    decode_id(global.0).map(|(slot, local)| (slot, MessageId(local)))
}

#[cfg(test)]
include!("slots.tests.rs");
