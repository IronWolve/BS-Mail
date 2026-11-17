//! Web addresses in the message being read: a click on one asks for it to be
//! opened in the user's browser, a right click opens the menu with Open and
//! Copy link (see `menus`), and the pointer turns into a hand over one. The
//! screen only asks: the program opens the address (`take_link_requests`) and
//! reports how that went.

use surface::Modifiers;

use crate::app_view::AppView;
use crate::layout::Layout;
use crate::view_state::Focus;
use crate::widgets::is_web_address;

/// The start of the note shown while an address is being opened.
const OPENING: &str = "Opening ";

impl AppView {
    /// The web address under a cell of the reader's body, when there is one,
    /// whole even where the address is cut over lines. The same answer the
    /// light on a link gets (`link_hit_at`).
    pub(crate) fn link_under(&self, layout: &Layout, col: u16, row: u16) -> Option<String> {
        self.link_hit_at(layout, col, row).map(|hit| hit.address)
    }

    /// A press in the reader: it takes focus. On the find box it is the box's;
    /// on the text it starts a selection, and on an address it also remembers
    /// the address, which the release opens when the pointer has not moved
    /// away (a drag that starts on an address selects).
    pub(crate) fn click_reader(&mut self, layout: &Layout, (col, row): (u16, u16), modifiers: Modifiers, grid: (u16, u16)) {
        self.focus = Focus::Reader;
        if self.click_find_bar(layout, col, row, modifiers.shift, grid) || self.click_attachment(layout, (col, row)) {
            return;
        }
        let address = self.link_under(layout, col, row);
        self.begin_selection(layout, col, row);
        self.reading.pending_link = address;
    }

    /// Opens what a click on an address asks for: a web address in the
    /// browser, a mail address or a `mailto:` link in the composer.
    pub(crate) fn open_link(&mut self, address: String) {
        if is_web_address(&address) {
            self.mail.set_note(Some(format!("{OPENING}{address}")));
            self.link_requests.push(address);
        } else {
            self.open_mail_link(&address);
        }
    }

    /// Whether nothing is open over the mail, so a click on it is for the mail.
    pub(crate) fn mail_is_free(&self) -> bool {
        self.setup.is_none()
            && self.options.is_none()
            && self.composer.is_none()
            && self.picker.is_none()
            && self.ask.is_none()
            && self.search_bar.is_none()
            && self.delete_prompt.is_none()
            && self.menu.is_none()
            && !self.overlays.is_open()
    }

    /// The addresses the user clicked, in order, once, for the program to open.
    pub fn take_link_requests(&mut self) -> Vec<String> {
        std::mem::take(&mut self.link_requests)
    }

    /// The browser took the address: the note that said it was being opened
    /// goes, unless something else has replaced it.
    pub fn link_opened(&mut self) {
        if self.mail.error.as_deref().is_some_and(|note| note.starts_with(OPENING)) {
            self.mail.set_note(None);
        }
    }

    /// No browser could open the address; the footer says why.
    pub fn link_open_failed(&mut self, reason: &str) {
        self.mail.set_note(Some(reason.to_string()));
    }
}

#[cfg(test)]
include!("links.tests.rs");
