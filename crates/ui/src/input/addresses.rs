//! The address book as the screen keeps it: addresses seen in the lists that
//! load and in the mail that is sent go into the book, which the composer
//! completes from, and the program saves the book when it changed.
//!
//! The book is a file in the data folder that the program reads and writes
//! (`addresses.txt`, see `composer::address_book`); here it is only text in
//! and text out.

use model::{Address, Event};

use crate::app_view::AppView;
use crate::composer::AddressBook;

impl AppView {
    /// Puts in the book what was saved of it in an earlier run.
    pub fn set_address_book(&mut self, book: AddressBook) {
        self.addresses = book;
    }

    /// The book, for the composer to complete from.
    pub fn address_book(&self) -> &AddressBook {
        &self.addresses
    }

    /// The text of the book to save, once, when it changed since the last
    /// time this was asked.
    pub fn take_address_book_text(&mut self) -> Option<String> {
        self.addresses.take_text_if_changed()
    }

    /// A save of the book did not work: it counts as changed again.
    pub fn address_book_unsaved(&mut self) {
        self.addresses.mark_unsaved();
    }

    /// Whether `email` is the address of one of the program's own accounts,
    /// which the book has no use for.
    pub(crate) fn is_own_address(&self, email: &str) -> bool {
        self.mail.accounts.iter().any(|account| account.address.eq_ignore_ascii_case(email))
    }

    /// Takes the addresses of the messages a list event carries into the
    /// book: who they are from, to and copied to.
    pub(crate) fn note_addresses(&mut self, event: &Event) {
        let (Event::EnvelopePage { envelopes, .. }
        | Event::OlderEnvelopes { envelopes, .. }
        | Event::SearchResults { envelopes, .. }) = event
        else {
            return;
        };
        let own: Vec<String> = self.mail.accounts.iter().map(|account| account.address.clone()).collect();
        let seen = envelopes
            .iter()
            .flat_map(|envelope| envelope.from.iter().chain(&envelope.to).chain(&envelope.cc))
            .filter(|address| !own.iter().any(|mine| mine.eq_ignore_ascii_case(&address.email)));
        self.addresses.observe_seen(seen);
    }

    /// Keeps the addresses a message that was sent went to.
    pub(crate) fn note_sent_to<'a>(&mut self, addresses: impl Iterator<Item = &'a Address>) {
        let others: Vec<Address> = addresses.filter(|address| !self.is_own_address(&address.email)).cloned().collect();
        self.addresses.observe_sent(&others);
    }
}

#[cfg(test)]
include!("addresses.tests.rs");
