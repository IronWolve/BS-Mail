//! Where the user is on the Options screen: the page, what has the keyboard,
//! the account highlighted, the removal question and how far the page is
//! scrolled. Pure state: it knows nothing of drawing or of the program.

use super::control::Control;

/// The pages, in the order the list shows them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Accounts,
    Appearance,
    Data,
}

impl Page {
    pub const ALL: [Page; 3] = [Self::Accounts, Self::Appearance, Self::Data];

    pub fn label(self) -> &'static str {
        match self {
            Self::Accounts => "Accounts",
            Self::Appearance => "Appearance",
            Self::Data => "Data",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|page| *page == self).unwrap_or(0)
    }
}

/// What has the keyboard: the list of pages, or one control of the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stop {
    Pages,
    Control(Control),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionsState {
    pub page: Page,
    pub stop: Stop,
    /// The question "remove this account?" is showing in place of the page.
    pub confirming: bool,
    /// First row of the page that is shown.
    pub scroll: usize,
    /// The highlighted row of the account list.
    pub cursor: usize,
}

impl Default for OptionsState {
    fn default() -> Self {
        Self::new()
    }
}

impl OptionsState {
    pub fn new() -> Self {
        Self {
            page: Page::Accounts,
            stop: Stop::Pages,
            confirming: false,
            scroll: 0,
            cursor: 0,
        }
    }

    /// Shows another page, from its top. The keyboard goes back to the list.
    pub fn select_page(&mut self, page: Page) {
        if page != self.page {
            self.page = page;
            self.scroll = 0;
        }
        self.confirming = false;
        self.stop = Stop::Pages;
    }

    /// The previous or next page, stopping at the ends. Returns whether the
    /// page changed.
    pub fn move_page(&mut self, forward: bool) -> bool {
        let at = self.page.index();
        let to = if forward { (at + 1).min(Page::ALL.len() - 1) } else { at.saturating_sub(1) };
        let changed = to != at;
        if changed {
            self.select_page(Page::ALL[to]);
        }
        changed
    }

    /// Moves the keyboard one stop: the list of pages, then each control of
    /// the page, round and round. The removal question has only its own two
    /// buttons.
    pub fn step_stop(&mut self, forward: bool, controls: &[Control]) {
        let mut ring: Vec<Stop> = Vec::with_capacity(controls.len() + 1);
        if !self.confirming {
            ring.push(Stop::Pages);
        }
        ring.extend(controls.iter().map(|control| Stop::Control(*control)));
        let count = ring.len();
        if count == 0 {
            return;
        }
        let next = match (ring.iter().position(|stop| *stop == self.stop), forward) {
            (Some(at), true) => (at + 1) % count,
            (Some(at), false) => (at + count - 1) % count,
            (None, true) => 0,
            (None, false) => count - 1,
        };
        self.stop = ring[next];
    }

    /// Puts the keyboard on the first control of the page, if it has one.
    pub fn enter_page(&mut self, controls: &[Control]) -> bool {
        match controls.first() {
            Some(first) => {
                self.stop = Stop::Control(*first);
                true
            }
            None => false,
        }
    }

    /// Keeps the keyboard on something that exists, after the page changed
    /// under it (an account was removed, a button went away).
    pub fn fix(&mut self, controls: &[Control]) {
        let lost = match self.stop {
            Stop::Control(control) => !controls.contains(&control),
            Stop::Pages => self.confirming,
        };
        if !lost {
            return;
        }
        self.stop = if self.confirming && controls.contains(&Control::KeepAccount) {
            Stop::Control(Control::KeepAccount)
        } else {
            controls.first().map_or(Stop::Pages, |control| Stop::Control(*control))
        };
    }

    /// Shows the removal question, with the keyboard on "Keep it".
    pub fn begin_confirm(&mut self) {
        self.confirming = true;
        self.scroll = 0;
        self.stop = Stop::Control(Control::KeepAccount);
    }

    /// Back to the page, with the keyboard on "Remove account".
    pub fn cancel_confirm(&mut self) {
        self.confirming = false;
        self.scroll = 0;
        self.stop = Stop::Control(Control::RemoveAccount);
    }

    /// Moves the account highlight, stopping at the ends. Returns whether it
    /// moved.
    pub fn move_cursor(&mut self, forward: bool, accounts: usize) -> bool {
        let to = if forward { (self.cursor + 1).min(accounts.saturating_sub(1)) } else { self.cursor.saturating_sub(1) };
        let moved = to != self.cursor && accounts > 0;
        if moved {
            self.cursor = to;
        }
        moved
    }

    /// Keeps the highlight on an account that exists.
    pub fn clamp_cursor(&mut self, accounts: usize) {
        self.cursor = self.cursor.min(accounts.saturating_sub(1));
    }
}

#[cfg(test)]
include!("state.tests.rs");
