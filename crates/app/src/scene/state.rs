//! What a way of showing the screen starts from, and waiting for real mail.

use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use model::{MailBackend, MailDate};
use ui::AppView;

use crate::accounts::Live;
use crate::pump::pump;

/// How often a one-shot draw checks whether real mail has arrived.
const SETTLE_STEP: Duration = Duration::from_millis(50);
/// How long a one-shot draw waits for real mail before drawing what it has.
pub const SETTLE_LIMIT: Duration = Duration::from_secs(20);

pub struct Scene {
    pub view: AppView,
    pub backend: Box<dyn MailBackend>,
    /// Messages the mock holds; zero for a real account.
    pub messages: usize,
    /// The mock file, the account file or the accounts folder the mail comes
    /// from.
    pub source: PathBuf,
    /// What the stats call `source`.
    pub label: &'static str,
    /// Real mail from at least one account, not the mock and not an empty
    /// start.
    pub real: bool,
    /// How many accounts the run started with; `None` for mock mail.
    pub accounts: Option<usize>,
    /// What changes the accounts while the program runs; `None` for mock mail.
    pub live: Option<Live>,
    /// What the theme means by plain text on the plain background.
    pub base: (host::Rgb, host::Rgb),
    /// A clock that stands still, so a picture of mock data never changes
    /// with the day it is taken; `None` follows the system clock.
    pub fixed_now: Option<MailDate>,
    /// Text size in points and row height, from the command line or the
    /// settings.
    pub font: (u8, f32),
}

impl Scene {
    /// Takes everything the backend has ready and answers what the screen
    /// then asks for, so the first frame has data in it.
    pub fn prime(&mut self) {
        pump(&mut self.view, self.backend.as_mut());
    }

    /// For a draw that happens once: as `prime`, and for real mail keeps
    /// asking until the first folder and message have arrived, a failure is
    /// showing, or `limit` runs out. A window does not need this; it fills
    /// in as the mail comes.
    pub fn settle(&mut self, limit: Duration) {
        self.prime();
        if !self.real {
            return;
        }
        let deadline = Instant::now() + limit;
        while !self.is_settled() && Instant::now() < deadline {
            thread::sleep(SETTLE_STEP);
            self.prime();
        }
    }

    fn is_settled(&self) -> bool {
        // The form hides the mail; there is nothing to wait for.
        if self.view.is_setting_up() {
            return true;
        }
        let mail = self.view.mail();
        if mail.error.is_some() {
            return true;
        }
        if mail.page.is_none() {
            return false;
        }
        match self.view.selection().message {
            Some(id) => mail.body.as_ref().is_some_and(|(shown, _)| *shown == id),
            None => true,
        }
    }
}
