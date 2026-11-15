//! Which folder is the Inbox, Sent, Drafts, Archive, Spam or Trash: the
//! server's SPECIAL-USE attributes first, common names as a fallback.
//!
//! "Bin" is not among the names for the Trash. Servers that call their Trash
//! "Bin" mark it, and a person's own folder with that name must never become
//! the place where deleting removes mail for good.

use model::FolderRole;

use super::mutf7;
use super::raw_folder::RawFolder;

/// Roles that can be told apart by a name; the first folder to claim a role
/// keeps it.
const NAMED: [(FolderRole, &[&str]); 5] = [
    (FolderRole::Sent, &["sent", "sent items", "sent messages", "sent mail"]),
    (FolderRole::Drafts, &["drafts", "draft"]),
    (FolderRole::Archive, &["archive", "archives"]),
    (
        FolderRole::Spam,
        &["spam", "junk", "junk e-mail", "junk email", "bulk mail"],
    ),
    (
        FolderRole::Trash,
        &["trash", "deleted items", "deleted messages"],
    ),
];

fn attribute_role(folder: &RawFolder) -> Option<FolderRole> {
    let table = [
        ("\\sent", FolderRole::Sent),
        ("\\drafts", FolderRole::Drafts),
        ("\\trash", FolderRole::Trash),
        ("\\junk", FolderRole::Spam),
        ("\\archive", FolderRole::Archive),
    ];
    table
        .into_iter()
        .find(|(attribute, _)| folder.has_attribute(attribute))
        .map(|(_, role)| role)
}

/// One role per folder, in the order of `folders`. Each role except `Plain`
/// goes to at most one folder.
pub fn assign_roles(folders: &[RawFolder]) -> Vec<FolderRole> {
    let mut roles = vec![FolderRole::Plain; folders.len()];
    for (at, folder) in folders.iter().enumerate() {
        if folder.path.eq_ignore_ascii_case("INBOX") {
            roles[at] = FolderRole::Inbox;
        }
    }
    for (at, folder) in folders.iter().enumerate() {
        if let Some(role) = attribute_role(folder).filter(|r| !claimed(&roles, *r)) {
            roles[at] = role;
        }
    }
    claim_all_mail(folders, &mut roles);
    for (role, names) in NAMED {
        if claimed(&roles, role) {
            continue;
        }
        let found = folders.iter().enumerate().find(|(at, folder)| {
            roles[*at] == FolderRole::Plain
                && is_near_top(folder, folders)
                && names.contains(&mutf7::decode(folder.leaf()).to_lowercase().as_str())
        });
        if let Some((at, _)) = found {
            roles[at] = role;
        }
    }
    roles
}

fn claimed(roles: &[FolderRole], role: FolderRole) -> bool {
    roles.contains(&role)
}

/// An "all mail" folder stands in for the archive when none is marked.
fn claim_all_mail(folders: &[RawFolder], roles: &mut [FolderRole]) {
    if claimed(roles, FolderRole::Archive) {
        return;
    }
    let all = folders
        .iter()
        .position(|folder| folder.has_attribute("\\all"));
    if let Some(at) = all.filter(|at| roles[*at] == FolderRole::Plain) {
        roles[at] = FolderRole::Archive;
    }
}

/// A name-based guess applies at the top level and directly under INBOX
/// (servers that put everything below it), nowhere deeper.
fn is_near_top(folder: &RawFolder, all: &[RawFolder]) -> bool {
    match folder.parent_path() {
        None => true,
        Some(parent) => {
            parent.eq_ignore_ascii_case("INBOX") && all.iter().any(|f| f.path == parent)
        }
    }
}

#[cfg(test)]
include!("folder_roles.tests.rs");
