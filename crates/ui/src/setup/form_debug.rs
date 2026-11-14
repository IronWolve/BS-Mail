//! What the form shows when it is printed for debugging: never the password
//! or the client secret, and never the text of a field the user typed.

use std::fmt;

use super::form::SetupForm;

impl fmt::Debug for SetupForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SetupForm")
            .field("signin", &self.signin)
            .field("focus", &self.focus)
            .field("waiting", &self.waiting)
            .field("allow_changes", &self.allow_changes)
            .field("editing", &self.edit_id)
            .field("password", &"<hidden>")
            .field("client_secret", &"<hidden>")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
include!("form_debug.tests.rs");
