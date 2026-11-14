//! Turning the message ids of a changing command into exact server
//! addresses: a folder, and UIDs inside it. An id that does not resolve is
//! set aside, never guessed at.

use model::{FolderId, MessageId, Uid};

use crate::imap_session::{MAX_PER_COMMAND, UidSet};
use crate::session_state::SessionState;

/// The messages of one folder that a command names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Group {
    pub folder: FolderId,
    /// The server's name for the folder.
    pub path: String,
    /// Each message with the UID it had when its id was issued.
    pub items: Vec<(MessageId, Uid)>,
}

/// What a list of ids came to.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Resolved {
    /// One group per folder, in the order the folders first appear.
    pub groups: Vec<Group>,
    /// Ids that name no known message.
    pub unknown: Vec<MessageId>,
}

impl Group {
    /// The messages in lists short enough for one command each, with the
    /// exact UIDs of every list.
    pub(super) fn batches(&self) -> Vec<(Vec<MessageId>, UidSet)> {
        self.items
            .chunks(MAX_PER_COMMAND)
            .filter_map(|chunk| {
                let set = UidSet::new(chunk.iter().map(|(_, uid)| uid.0))?;
                Some((chunk.iter().map(|(id, _)| *id).collect(), set))
            })
            .collect()
    }
}

/// Looks every id up. A repeated id counts once; an id whose folder is no
/// longer in the listing is unknown.
pub(super) fn resolve(state: &SessionState, ids: &[MessageId]) -> Resolved {
    let mut out = Resolved::default();
    let mut seen: Vec<MessageId> = Vec::new();
    for &id in ids {
        if seen.contains(&id) {
            continue;
        }
        seen.push(id);
        let found = state.message_key(id).and_then(|(folder, uid)| {
            state.folder_path(folder).map(|path| (folder, path.to_owned(), uid))
        });
        let Some((folder, path, uid)) = found else {
            out.unknown.push(id);
            continue;
        };
        match out.groups.iter_mut().find(|group| group.folder == folder) {
            Some(group) => group.items.push((id, uid)),
            None => out.groups.push(Group { folder, path, items: vec![(id, uid)] }),
        }
    }
    out
}

#[cfg(test)]
include!("resolve.tests.rs");
