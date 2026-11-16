//! Opening what the user asked the browser for: the web addresses clicked in
//! a message, and the pages of HTML messages (see `pages`). Each goes to the
//! opener, and how that went is told to the footer.

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
        changed |= self.page_steps();
        match self.links.poll() {
            Some(Ok(())) => match self.page_removed.take() {
                Some(removed) => self.view.html_opened(removed),
                None => self.view.link_opened(),
            },
            Some(Err(reason)) => {
                self.page_removed = None;
                self.view.link_open_failed(&reason);
            }
            None => return changed,
        }
        true
    }
}

#[cfg(test)]
include!("links.tests.rs");
