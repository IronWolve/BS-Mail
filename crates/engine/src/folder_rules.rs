//! What path a new or renamed folder gets, kept apart from the server so it
//! can be tested on its own and worded once for the real backend and the mock.
//! (Which folders cannot be changed is `model::folder_rules`.)

use model::{Folder, check_folder_name};

use crate::imap_session::{decode_mailbox_name, encode_mailbox_name};

/// The server's path for a folder called `name` inside the folder at
/// `parent` (the top level when there is none): the name checked, kept clear
/// of the separator, put in the server's encoding and joined to the parent.
/// `existing` are the folders the server has, so a name already taken is
/// refused here, in words, rather than by the server.
pub(crate) fn new_path(
    parent: Option<&str>,
    separator: Option<&str>,
    name: &str,
    existing: &[Folder],
) -> Result<String, String> {
    let name = check_folder_name(name)?;
    let separator = separator.filter(|sep| !sep.is_empty());
    if let Some(sep) = separator.filter(|sep| name.contains(sep)) {
        return Err(format!(
            "a folder name cannot hold {sep}, which this server uses to put one folder inside another; \
             make the folder inside the one you want instead"
        ));
    }
    let encoded = encode_mailbox_name(&name);
    let path = match (parent, separator) {
        (None, _) if name.eq_ignore_ascii_case("INBOX") => {
            return Err("INBOX is the Inbox, which every account has already".to_owned());
        }
        (None, _) => encoded,
        (Some(parent), Some(sep)) => format!("{parent}{sep}{encoded}"),
        (Some(_), None) => return Err("this server has no folders inside folders".to_owned()),
    };
    if existing.iter().any(|folder| folder.path.eq_ignore_ascii_case(&path)) {
        return Err(format!("a folder named {name} is already there"));
    }
    Ok(path)
}

/// The path a folder at `path` gets when it is renamed to `name`: the same
/// place in the tree, the new last segment.
pub(crate) fn renamed_path(
    path: &str,
    separator: Option<&str>,
    name: &str,
    existing: &[Folder],
) -> Result<String, String> {
    let sep = separator.filter(|sep| !sep.is_empty());
    let parent = sep.and_then(|sep| path.rfind(sep).map(|at| &path[..at]));
    let last = sep.map_or(path, |sep| path.rsplit(sep).next().unwrap_or(path));
    if decode_mailbox_name(last) == name.trim() {
        return Err("that is already its name".to_owned());
    }
    new_path(parent, separator, name, existing)
}

#[cfg(test)]
include!("folder_rules.tests.rs");
