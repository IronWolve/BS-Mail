//! A try at an account that is in flight.

use model::{Event, MailBackend};

use crate::accounts::Plan;
use crate::browser_login::BrowserLogin;

/// One try: where the account will be saved, and how far it has got.
pub(super) struct Attempt {
    pub plan: Plan,
    pub stage: Stage,
}

pub(super) enum Stage {
    /// Waiting for the user to finish in the browser. Dropping the login
    /// stops listening for it.
    Browser(Box<dyn BrowserLogin>),
    /// The account is connecting. What it says is kept, in order, for when it
    /// joins the running accounts.
    Trial { backend: Box<dyn MailBackend>, kept: Vec<Event> },
}
