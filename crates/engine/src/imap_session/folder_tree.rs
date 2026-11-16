//! Turns the server's flat mailbox list into the sidebar's tree: roles,
//! parents from the hierarchy delimiter, and a depth-first order. The two
//! saved searches, Starred and Alerts, are rows of the tree too: they are not
//! mailboxes, so their paths are names no server can have (they hold the list
//! wildcard `*`) and are never sent to one.

use std::collections::HashMap;

use model::{AccountId, Folder, FolderId, FolderRole};

use super::folder_roles::{Unmatched, assign_roles_named};
use super::mutf7;
use super::raw_folder::RawFolder;
use crate::account_folders::FolderPaths;

/// The path of the Starred row, which no mailbox can have.
pub const STARRED_PATH: &str = "*starred*";

/// The path of the Alerts row.
pub const ALERTS_PATH: &str = "*alerts*";

/// The folder list and what the account file named that is not on the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    pub folders: Vec<Folder>,
    pub missing: Vec<Unmatched>,
}

/// Builds the folder list. `id_for` gives the stable id of a mailbox name.
/// `named` is what the account file says about the roles. Roots come in the
/// order Inbox, Starred, Sent, Drafts, Archive, the rest by name, Alerts,
/// Spam, Trash; children follow their parent, by name.
pub fn build_tree(
    account: &AccountId,
    listed: Vec<RawFolder>,
    named: &FolderPaths,
    id_for: &mut dyn FnMut(&str) -> FolderId,
) -> Tree {
    let mut raw = prune(listed);
    let (mut roles, missing) = assign_roles_named(&raw, named);
    add_saved_searches(&mut raw, &mut roles);
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
    Tree { folders: out, missing }
}

/// Adds the Starred and Alerts rows, each unless a mailbox of that name is
/// listed already (the sidebar would then show two of the same name).
fn add_saved_searches(raw: &mut Vec<RawFolder>, roles: &mut Vec<FolderRole>) {
    for (path, role, name) in [(STARRED_PATH, FolderRole::Starred, "starred"), (ALERTS_PATH, FolderRole::Alerts, "alerts")] {
        if raw.iter().any(|folder| mutf7::decode(folder.leaf()).eq_ignore_ascii_case(name)) {
            continue;
        }
        raw.push(RawFolder { path: path.to_owned(), delimiter: None, attributes: Vec::new(), unread: 0, total: 0 });
        roles.push(role);
    }
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
    match role {
        FolderRole::Inbox if raw.path.eq_ignore_ascii_case("INBOX") => "Inbox".to_owned(),
        FolderRole::Starred => "Starred".to_owned(),
        FolderRole::Alerts => "Alerts".to_owned(),
        _ => mutf7::decode(raw.leaf()),
    }
}

fn sort_key(folder: &RawFolder, role: FolderRole) -> (u8, String, String) {
    let rank = match role {
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
