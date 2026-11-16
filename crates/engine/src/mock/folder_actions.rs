//! The folder actions on the mock's in-memory data: making, renaming and
//! deleting a folder, marking a folder read, emptying the Trash or Spam, and
//! setting a tag. What the real backend refuses is refused here in the same
//! words (see `folder_rules` and `change_refusals`), so a test against the
//! mock sees the refusal the real account gives.

use model::{AccountId, Envelope, FlagChange, Folder, FolderId, FolderRole, MessageId, check_tag, holds_folders, not_emptiable, protected, saved_search_refusal};

use super::fixture_load::Fixture;
use super::mutate::{adjust_counts, apply_flags};
use crate::folder_rules::{new_path, renamed_path};

/// The separator the mock's paths use between a folder and the one inside it.
const SEPARATOR: &str = "/";

/// The folder with this id when it is a mailbox, or the refusal.
fn mailbox(data: &Fixture, id: FolderId) -> Result<&Folder, String> {
    match data.folders.iter().find(|folder| folder.id == id) {
        None => Err(format!("unknown folder id: {}", id.0)),
        Some(folder) => saved_search_refusal(folder).map_or(Ok(folder), Err),
    }
}

/// Makes a folder called `name` inside `parent`, or at the top.
pub(crate) fn create_folder(
    data: &mut Fixture,
    account: &AccountId,
    parent: Option<FolderId>,
    name: &str,
) -> Result<(), String> {
    if *account != data.account {
        return Err(format!("unknown account: {}", account.0));
    }
    let inside = parent.map(|id| mailbox(data, id).cloned()).transpose()?;
    let path = new_path(inside.as_ref().map(|folder| folder.path.as_str()), Some(SEPARATOR), name, &data.folders)?;
    let id = FolderId(data.folders.iter().map(|folder| folder.id.0).max().unwrap_or(0) + 1);
    let folder = Folder {
        id,
        account: data.account.clone(),
        path,
        name: name.trim().to_owned(),
        parent,
        role: FolderRole::Plain,
        unread: 0,
        total: 0,
        expanded: false,
    };
    let at = insertion_point(&data.folders, parent, &folder.name);
    data.folders.insert(at, folder);
    if let Some(parent) = data.folders.iter_mut().find(|folder| Some(folder.id) == parent) {
        parent.expanded = true;
    }
    Ok(())
}

/// Where a new folder goes in the depth-first list: among its siblings in the
/// order the real tree uses (the special folders that come first, the plain
/// ones by name, then Alerts, Spam and Trash).
fn insertion_point(folders: &[Folder], parent: Option<FolderId>, name: &str) -> usize {
    let rank = |folder: &Folder| match folder.role {
        FolderRole::Inbox => 0,
        FolderRole::Starred => 1,
        FolderRole::Sent => 2,
        FolderRole::Drafts => 3,
        FolderRole::Archive => 4,
        FolderRole::Plain => 5,
        FolderRole::Alerts => 6,
        FolderRole::Spam => 7,
        FolderRole::Trash => 8,
    };
    let key = |folder: &Folder| (rank(folder), if folder.role == FolderRole::Plain { folder.name.to_lowercase() } else { String::new() });
    let mine = (5, name.to_lowercase());
    let next = folders.iter().filter(|folder| folder.parent == parent).find(|folder| key(folder) > mine);
    match next {
        Some(next) => folders.iter().position(|folder| folder.id == next.id).unwrap_or(folders.len()),
        None => end_of_branch(folders, parent),
    }
}

/// The index just after `parent` and everything inside it; the end of the
/// list for the top level.
fn end_of_branch(folders: &[Folder], parent: Option<FolderId>) -> usize {
    let Some(parent) = parent else {
        return folders.len();
    };
    let mut at = folders.iter().position(|folder| folder.id == parent).map_or(folders.len(), |index| index + 1);
    while folders.get(at).is_some_and(|folder| inside(folders, folder, parent)) {
        at += 1;
    }
    at
}

/// Whether `folder` is `ancestor` or somewhere inside it.
fn inside(folders: &[Folder], folder: &Folder, ancestor: FolderId) -> bool {
    let mut next = folder.parent;
    for _ in 0..folders.len() {
        match next {
            Some(id) if id == ancestor => return true,
            Some(id) => next = folders.iter().find(|candidate| candidate.id == id).and_then(|candidate| candidate.parent),
            None => return false,
        }
    }
    false
}

/// Gives a folder another name; the folders inside it follow.
pub(crate) fn rename_folder(data: &mut Fixture, id: FolderId, name: &str) -> Result<(), String> {
    let found = mailbox(data, id)?.clone();
    if let Some(reason) = protected(&found, "renamed") {
        return Err(reason);
    }
    let to = renamed_path(&found.path, Some(SEPARATOR), name, &data.folders)?;
    for folder in &mut data.folders {
        if folder.id == id {
            folder.name = name.trim().to_owned();
            folder.path.clone_from(&to);
        } else if let Some(rest) = folder.path.strip_prefix(&format!("{}{SEPARATOR}", found.path)) {
            folder.path = format!("{to}{SEPARATOR}{rest}");
        }
    }
    Ok(())
}

/// Removes a folder and every message in it.
pub(crate) fn delete_folder(data: &mut Fixture, id: FolderId) -> Result<(), String> {
    let found = mailbox(data, id)?.clone();
    if let Some(reason) = protected(&found, "deleted").or_else(|| holds_folders(&data.folders, id, &found.name)) {
        return Err(reason);
    }
    adjust_counts(data, id, -i64::from(found.unread), -i64::from(found.total));
    data.folders.retain(|folder| folder.id != id);
    data.messages.retain(|envelope| envelope.folder != id);
    Ok(())
}

/// Marks every unread message of the folder read. Returns the new flags of
/// the messages of the fixture that changed.
pub(crate) fn mark_all_read(data: &mut Fixture, id: FolderId) -> Result<Vec<(MessageId, model::Flags)>, String> {
    mailbox(data, id)?;
    let unread: Vec<MessageId> =
        data.messages.iter().filter(|envelope| envelope.folder == id && !envelope.flags.seen).map(|envelope| envelope.id).collect();
    let outcome = apply_flags(data, &unread, FlagChange { seen: Some(true), ..FlagChange::default() });
    // The folder's own count can be larger than the sample the fixture holds.
    let left = data.folders.iter().find(|folder| folder.id == id).map_or(0, |folder| folder.unread);
    adjust_counts(data, id, -i64::from(left), 0);
    Ok(outcome.updates)
}

/// Removes every message of a Trash or Spam folder.
pub(crate) fn empty_folder(data: &mut Fixture, id: FolderId) -> Result<(), String> {
    let found = mailbox(data, id)?.clone();
    if let Some(reason) = not_emptiable(&found) {
        return Err(reason);
    }
    adjust_counts(data, id, -i64::from(found.unread), -i64::from(found.total));
    data.messages.retain(|envelope| envelope.folder != id);
    Ok(())
}

/// What a tag change did.
pub(crate) struct TagOutcome {
    pub updates: Vec<(MessageId, Vec<String>)>,
    pub unknown: Vec<MessageId>,
}

/// Adds `tag` to each message, or takes it off.
pub(crate) fn set_tag(data: &mut Fixture, ids: &[MessageId], tag: &str, on: bool) -> Result<TagOutcome, String> {
    let tag = check_tag(tag)?;
    let mut outcome = TagOutcome { updates: Vec::new(), unknown: Vec::new() };
    for &id in ids {
        let Some(envelope) = data.messages.iter_mut().find(|envelope| envelope.id == id) else {
            outcome.unknown.push(id);
            continue;
        };
        retag(envelope, &tag, on);
        outcome.updates.push((id, envelope.tags.clone()));
    }
    Ok(outcome)
}

fn retag(envelope: &mut Envelope, tag: &str, on: bool) {
    let held = envelope.tags.iter().position(|have| have.eq_ignore_ascii_case(tag));
    match (held, on) {
        (None, true) => envelope.tags.push(tag.to_owned()),
        (Some(at), false) => {
            envelope.tags.remove(at);
        }
        _ => {}
    }
}

#[cfg(test)]
include!("folder_actions.tests.rs");
