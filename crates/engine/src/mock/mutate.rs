//! Changes the mock's in-memory data when commands arrive: flags, moves and
//! deletes, with the folder counts kept in step. A parent folder's counts
//! include its children (as in the fixture); the virtual Starred and Alerts
//! folders are not recounted. What the real backend refuses is refused here
//! too, in the same words (see `change_refusals`).

use model::{FlagChange, Flags, FolderId, FolderRole, MessageId};

use super::fixture_load::Fixture;
use crate::change_refusals::NO_TRASH;

/// What a flag change did.
pub(crate) struct FlagOutcome {
    /// The new flags of every message that was found.
    pub updates: Vec<(MessageId, Flags)>,
    /// Ids that are not in the fixture.
    pub unknown: Vec<MessageId>,
    /// Whether any folder's unread count moved.
    pub counts_moved: bool,
}

/// What a move or delete did.
pub(crate) struct MoveOutcome {
    pub unknown: Vec<MessageId>,
    /// Folders that lost or gained messages, each once.
    pub touched: Vec<FolderId>,
    /// Why the known messages were left alone, when they were.
    pub refused: Option<&'static str>,
}

/// Applies `change` to each message and adjusts unread counts when a message
/// turns read or unread.
pub(crate) fn apply_flags(
    data: &mut Fixture,
    ids: &[MessageId],
    change: FlagChange,
) -> FlagOutcome {
    let mut outcome = FlagOutcome {
        updates: Vec::new(),
        unknown: Vec::new(),
        counts_moved: false,
    };
    for &id in ids {
        let Some(envelope) = data.messages.iter_mut().find(|envelope| envelope.id == id) else {
            outcome.unknown.push(id);
            continue;
        };
        let was_seen = envelope.flags.seen;
        envelope.flags.apply(change);
        let (now_seen, folder) = (envelope.flags.seen, envelope.folder);
        outcome.updates.push((id, envelope.flags));
        if was_seen != now_seen {
            let delta = if now_seen { -1 } else { 1 };
            adjust_counts(data, folder, delta, 0);
            outcome.counts_moved = true;
        }
    }
    outcome
}

/// Moves each message into `to`.
pub(crate) fn move_messages(data: &mut Fixture, ids: &[MessageId], to: FolderId) -> MoveOutcome {
    relocate(data, ids, |_| Some(to))
}

/// Deletes each message: into the Trash folder, or for good when it is
/// already there. An account with no Trash folder refuses and removes nothing.
pub(crate) fn delete_messages(data: &mut Fixture, ids: &[MessageId]) -> MoveOutcome {
    let trash = data
        .folders
        .iter()
        .find(|folder| folder.role == FolderRole::Trash)
        .map(|folder| folder.id);
    let Some(trash) = trash else {
        return refuse(data, ids, NO_TRASH);
    };
    relocate(data, ids, |source| Some(trash).filter(|&trash| trash != source))
}

/// Leaves every message where it is; ids that are not in the fixture are
/// still told apart.
fn refuse(data: &Fixture, ids: &[MessageId], reason: &'static str) -> MoveOutcome {
    let unknown: Vec<MessageId> = ids
        .iter()
        .copied()
        .filter(|id| !data.messages.iter().any(|envelope| envelope.id == *id))
        .collect();
    let refused = (unknown.len() < ids.len()).then_some(reason);
    MoveOutcome {
        unknown,
        touched: Vec::new(),
        refused,
    }
}

/// Moves each message to the folder `target` picks for it (`None` removes it).
fn relocate(
    data: &mut Fixture,
    ids: &[MessageId],
    target: impl Fn(FolderId) -> Option<FolderId>,
) -> MoveOutcome {
    let mut outcome = MoveOutcome {
        unknown: Vec::new(),
        touched: Vec::new(),
        refused: None,
    };
    for &id in ids {
        let Some(index) = data.messages.iter().position(|envelope| envelope.id == id) else {
            outcome.unknown.push(id);
            continue;
        };
        let source = data.messages[index].folder;
        let unread = i64::from(!data.messages[index].flags.seen);
        adjust_counts(data, source, -unread, -1);
        touch(&mut outcome.touched, source);
        match target(source) {
            Some(to) => {
                data.messages[index].folder = to;
                adjust_counts(data, to, unread, 1);
                touch(&mut outcome.touched, to);
            }
            None => {
                data.messages.remove(index);
            }
        }
    }
    outcome
}

fn touch(touched: &mut Vec<FolderId>, folder: FolderId) {
    if !touched.contains(&folder) {
        touched.push(folder);
    }
}

/// Shifts a folder's counts and those of its ancestors, never below zero.
pub(super) fn adjust_counts(data: &mut Fixture, folder: FolderId, unread: i64, total: i64) {
    let mut next = Some(folder);
    // The bound keeps a malformed (cyclic) tree from looping forever.
    for _ in 0..data.folders.len() {
        let Some(current) = next else { break };
        let Some(entry) = data.folders.iter_mut().find(|entry| entry.id == current) else {
            break;
        };
        entry.unread = shifted(entry.unread, unread);
        entry.total = shifted(entry.total, total);
        next = entry.parent;
    }
}

fn shifted(value: u32, delta: i64) -> u32 {
    u32::try_from((i64::from(value) + delta).max(0)).unwrap_or(u32::MAX)
}
