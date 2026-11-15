//! Opening the web addresses the user clicked in a message: each goes to the
//! browser opener, and how that went is told to the footer.

use super::Screen;

impl Screen {
    /// Hands the clicked addresses to the opener and reports its answer.
    /// Returns whether anything changed.
    pub(super) fn link_steps(&mut self) -> bool {
        let mut changed = false;
        for address in self.view.take_link_requests() {
            self.links.open(&address);
            changed = true;
        }
        match self.links.poll() {
            Some(Ok(())) => self.view.link_opened(),
            Some(Err(reason)) => self.view.link_open_failed(&reason),
            None => return changed,
        }
        true
    }
}

#[cfg(test)]
include!("links.tests.rs");
