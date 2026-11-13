//! Turns the server's flat mailbox list into the sidebar's tree: roles,
//! parents from the hierarchy delimiter, and a depth-first order.

use std::collections::HashMap;

use model::{AccountId, Folder, FolderId, FolderRole};

use super::folder_roles::assign_roles;
use super::mutf7;
use super::raw_folder::RawFolder;

/// Builds the folder list. `id_for` gives the stable id of a mailbox name.
/// Roots come in the order Inbox, Sent, Drafts, Archive, the rest by name,
/// Spam, Trash; children follow their parent, by name.
pub fn build_tree(
    account: &AccountId,
    listed: Vec<RawFolder>,
    id_for: &mut dyn FnMut(&str) -> FolderId,
) -> Vec<Folder> {
    let raw = prune(listed);
    let roles = assign_roles(&raw);
    let parents = find_parents(&raw);
    let mut children: HashMap<Option<usize>, Vec<usize>> = HashMap::new();
    for (at, parent) in parents.iter().enumerate() {
        children.entry(*parent).or_default().push(at);
    }
    for siblings in children.values_mut() {
        siblings.sort_by_key(|&at| sort_key(&raw[at], roles[at]));
    }
    let mut out = Vec::with_capacity(raw.len());
    let mut stack: Vec<(usize, Option<FolderId>)> = children
        .get(&None)
        .map(|roots| roots.iter().rev().map(|&at| (at, None)).collect())
        .unwrap_or_default();
    while let Some((at, parent)) = stack.pop() {
        let id = id_for(&raw[at].path);
        let kids = children.get(&Some(at)).map(Vec::as_slice).unwrap_or(&[]);
        out.push(folder_row(account, &raw[at], roles[at], id, parent, !kids.is_empty()));
        stack.extend(kids.iter().rev().map(|&kid| (kid, Some(id))));
    }
    out
}

fn folder_row(
    account: &AccountId,
    raw: &RawFolder,
    role: FolderRole,
    id: FolderId,
    parent: Option<FolderId>,
    has_children: bool,
) -> Folder {
    Folder {
        id,
        account: account.clone(),
        path: raw.path.clone(),
        name: display_name(raw, role),
        parent,
        role,
        unread: raw.unread,
        total: raw.total,
        expanded: has_children,
    }
}

/// The name the sidebar shows. The top-level mailbox every server calls
/// INBOX is shown as "Inbox", as the rest of the screen writes it; its path
/// is left alone, because that is what the server is asked for.
fn display_name(raw: &RawFolder, role: FolderRole) -> String {
    if role == FolderRole::Inbox && raw.path.eq_ignore_ascii_case("INBOX") {
        return "Inbox".to_owned();
    }
    mutf7::decode(raw.leaf())
}

fn sort_key(folder: &RawFolder, role: FolderRole) -> (u8, String, String) {
    let rank = match role {
        FolderRole::Inbox => 0,
        FolderRole::Sent => 1,
        FolderRole::Drafts => 2,
        FolderRole::Archive => 3,
        FolderRole::Spam => 5,
        FolderRole::Trash => 6,
        FolderRole::Plain | FolderRole::Starred | FolderRole::Alerts => 4,
    };
    (
        rank,
        mutf7::decode(folder.leaf()).to_lowercase(),
        folder.path.clone(),
    )
}

/// Drops mailboxes that do not exist, and containers that cannot be opened
/// and hold nothing worth showing.
fn prune(listed: Vec<RawFolder>) -> Vec<RawFolder> {
    let mut raw: Vec<RawFolder> = listed
        .into_iter()
        .filter(|folder| !folder.has_attribute("\\nonexistent"))
        .collect();
    loop {
        let parents = find_parents(&raw);
        let has_child: Vec<bool> = (0..raw.len()).map(|at| parents.contains(&Some(at))).collect();
        let before = raw.len();
        let mut at = 0;
        raw.retain(|folder| {
            let keep = !folder.has_attribute("\\noselect") || has_child[at];
            at += 1;
            keep
        });
        if raw.len() == before {
            return raw;
        }
    }
}

/// For each folder, the index of its nearest listed ancestor.
fn find_parents(raw: &[RawFolder]) -> Vec<Option<usize>> {
    let index: HashMap<&str, usize> = raw
        .iter()
        .enumerate()
        .map(|(at, folder)| (folder.path.as_str(), at))
        .collect();
    raw.iter()
        .map(|folder| {
            let delimiter = folder.delimiter.as_deref().filter(|d| !d.is_empty())?;
            let mut path = folder.parent_path()?;
            loop {
                if let Some(&at) = index.get(path) {
                    return Some(at);
                }
                path = path.rfind(delimiter).map(|cut| &path[..cut])?;
            }
        })
        .collect()
}

#[cfg(test)]
include!("folder_tree.tests.rs");
