//! The rules that decide what a change may do, kept apart from the server
//! so each one can be tested on its own.

use model::{FlagChange, Flags, FolderId, FolderRole};

use crate::imap_session::{ServerCaps, StoreFlag, StoreMode};
use crate::session_state::SessionState;

/// Why the deleted flag in a flag change is not carried out.
pub(super) const DELETED_NOT_HERE: &str =
    "the deleted flag is not changed this way; use delete";

/// Why a move is refused for a server with neither MOVE nor UIDPLUS.
const CANNOT_MOVE: &str = "this server cannot move safely (it offers neither MOVE nor UIDPLUS); \
     nothing was moved";

/// What a flag change asks of the server: flags to add and flags to remove.
/// `\Deleted` is never among them.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct FlagPlan {
    pub add: Vec<StoreFlag>,
    pub remove: Vec<StoreFlag>,
    /// The change named the deleted flag, which is ignored.
    pub deleted_ignored: bool,
}

impl FlagPlan {
    /// True when there is nothing to send.
    pub(super) fn is_empty(&self) -> bool {
        self.add.is_empty() && self.remove.is_empty()
    }

    /// The commands to send, in order: one to add, one to remove.
    pub(super) fn steps(&self) -> Vec<(StoreMode, &[StoreFlag])> {
        let mut steps = Vec::new();
        if !self.add.is_empty() {
            steps.push((StoreMode::Add, self.add.as_slice()));
        }
        if !self.remove.is_empty() {
            steps.push((StoreMode::Remove, self.remove.as_slice()));
        }
        steps
    }

    /// True when `flags` are what the change asked for.
    pub(super) fn satisfied_by(&self, flags: Flags) -> bool {
        self.add.iter().all(|flag| flag_is_set(flags, *flag))
            && self.remove.iter().all(|flag| !flag_is_set(flags, *flag))
    }
}

fn flag_is_set(flags: Flags, flag: StoreFlag) -> bool {
    match flag {
        StoreFlag::Seen => flags.seen,
        StoreFlag::Flagged => flags.flagged,
        StoreFlag::Answered => flags.answered,
        StoreFlag::Draft => flags.draft,
        StoreFlag::Important => flags.alert,
    }
}

/// Splits a flag change into what to add and what to remove.
pub(super) fn plan_flags(change: FlagChange) -> FlagPlan {
    let mut plan = FlagPlan {
        deleted_ignored: change.deleted.is_some(),
        ..FlagPlan::default()
    };
    let wanted = [
        (StoreFlag::Seen, change.seen),
        (StoreFlag::Flagged, change.flagged),
        (StoreFlag::Answered, change.answered),
        (StoreFlag::Draft, change.draft),
        (StoreFlag::Important, change.alert),
    ];
    for (flag, value) in wanted {
        match value {
            Some(true) => plan.add.push(flag),
            Some(false) => plan.remove.push(flag),
            None => {}
        }
    }
    plan
}

/// How to take messages out of a folder into another one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MoveHow {
    /// One `UID MOVE`.
    Native,
    /// Copy, then mark and expunge exactly the originals.
    ViaCopy,
    /// The server offers no safe way.
    Refuse(&'static str),
}

pub(super) fn move_how(caps: &ServerCaps) -> MoveHow {
    if caps.move_ext {
        MoveHow::Native
    } else if caps.uidplus {
        MoveHow::ViaCopy
    } else {
        MoveHow::Refuse(CANNOT_MOVE)
    }
}

/// The folder a move goes to: its id and the server's name for it, or the
/// reason it cannot be a destination.
pub(super) fn destination(state: &SessionState, to: FolderId) -> Result<(FolderId, String), String> {
    let Some(folder) = state.folder(to) else {
        return Err(format!("unknown folder id: {}", to.0));
    };
    if matches!(folder.role, FolderRole::Starred | FolderRole::Alerts) {
        return Err("that folder is a saved search, not a mailbox; nothing was moved".to_owned());
    }
    Ok((folder.id, folder.path.clone()))
}

/// What a delete does with the messages of one folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum DeletePlan {
    /// Remove them for good: they are already in the Trash folder.
    Forever,
    /// Move them to the Trash folder.
    ToTrash {
        trash: FolderId,
        path: String,
        how: MoveHow,
    },
    Refuse(String),
}

pub(super) fn plan_delete(state: &SessionState, source: FolderId, caps: &ServerCaps) -> DeletePlan {
    let in_trash = state
        .folder(source)
        .is_some_and(|folder| folder.role == FolderRole::Trash);
    if in_trash {
        return if caps.uidplus {
            DeletePlan::Forever
        } else {
            DeletePlan::Refuse(
                "this server cannot delete safely (it does not offer UIDPLUS); \
                 nothing was deleted"
                    .to_owned(),
            )
        };
    }
    let Some(trash) = state.trash_folder() else {
        return DeletePlan::Refuse("this account has no Trash folder; nothing was deleted".to_owned());
    };
    match move_how(caps) {
        MoveHow::Refuse(_) => DeletePlan::Refuse(
            "this server cannot move mail to the Trash safely (it offers neither MOVE \
             nor UIDPLUS); nothing was deleted"
                .to_owned(),
        ),
        how => DeletePlan::ToTrash {
            trash: trash.id,
            path: trash.path.clone(),
            how,
        },
    }
}

#[cfg(test)]
include!("change_rules.tests.rs");
