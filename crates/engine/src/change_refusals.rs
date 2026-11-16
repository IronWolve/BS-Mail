//! What a change to mail is refused for, worded once for the real backend and
//! the mock, so a test against the mock sees the refusal the real account
//! gives. The rules that need the server (what it can do) stay in the real
//! backend; only what the folder roles and the flags decide is here.

use model::FolderRole;

/// Why the deleted flag in a flag change is not carried out.
pub(crate) const DELETED_NOT_HERE: &str = "the deleted flag is not changed this way; use delete";

/// Why a delete is refused on an account with no Trash folder. It never
/// becomes a removal for good. It names the key of the account file that says
/// which folder is the Trash.
pub(crate) const NO_TRASH: &str = "this account has no Trash folder; nothing was deleted. \
     Name one in the account file: folders = { trash = \"<the folder's path on the server>\" }";

/// Why a move to a Starred or Alerts folder is refused.
pub(crate) const SAVED_SEARCH: &str = "that folder is a saved search, not a mailbox; nothing was moved";

/// The reason a folder with this role cannot be the destination of a move, or
/// `None` when it can.
pub(crate) fn move_refusal(role: FolderRole) -> Option<&'static str> {
    matches!(role, FolderRole::Starred | FolderRole::Alerts).then_some(SAVED_SEARCH)
}

#[cfg(test)]
include!("change_refusals.tests.rs");
