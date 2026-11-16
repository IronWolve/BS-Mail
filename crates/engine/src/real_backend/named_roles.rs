//! Telling the person, once, that the account file names a folder the server
//! does not have: the role is then guessed as if it were not named. The
//! sentence is short because the footer holds one line.

use super::worker::Worker;
use crate::account_folders::key_of;
use crate::imap_session::UnmatchedRole;

impl Worker {
    /// Sends one failure notice named `folders` for the folders of the account
    /// file that were not found, the first time a listing shows some. Later
    /// listings, and a reconnect, say nothing more.
    pub(super) fn tell_unmatched_roles(&mut self, unmatched: &[UnmatchedRole]) {
        if unmatched.is_empty() || self.roles_told {
            return;
        }
        self.roles_told = true;
        let list: Vec<String> = unmatched.iter().map(|(role, path)| format!("{} = \"{path}\"", key_of(*role))).collect();
        let reason = format!("the account file names folders the server lacks: {}", list.join(", "));
        self.sink.send(self.sink.failure("folders", reason));
    }
}

#[cfg(test)]
include!("named_roles.tests.rs");
