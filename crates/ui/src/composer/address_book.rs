//! The addresses the program has seen, for completing a recipient as it is
//! typed: those in the lists it has loaded and those it has sent mail to.
//!
//! The book is kept in memory here; the program saves it as one private file
//! in the data folder, `addresses.txt`, one `Name <address>` per line, most
//! recent first, and reads it back at the next start. Nothing in it is a
//! path. It holds at most `MAX_ENTRIES` addresses (the least recent go first)
//! and each address once, whatever its capitals.

use std::collections::HashSet;

use model::Address;

use super::address_list::{format, parse};

/// The most addresses kept. The oldest go when more come.
pub const MAX_ENTRIES: usize = 2000;

/// The longest display name kept; a longer one is dropped and the address
/// stays.
const MAX_NAME_CHARS: usize = 100;

#[derive(Debug, Clone, Default)]
pub struct AddressBook {
    /// Most recent first.
    entries: Vec<Address>,
    /// The addresses in `entries`, lowercased.
    known: HashSet<String>,
    /// The addresses of `entries` that have no name, lowercased, so a name
    /// seen later can be given to them without looking at every entry.
    unnamed: HashSet<String>,
    /// Something changed since the text was last taken for saving.
    changed: bool,
}

impl AddressBook {
    /// The book a saved text holds. A line that is not one address is left
    /// out; the rest keep their order, and what goes past the cap is dropped.
    pub fn from_text(text: &str) -> Self {
        let mut book = Self::default();
        for line in text.lines() {
            let Some(address) = parse(line).ok().and_then(|mut found| (found.len() == 1).then(|| found.remove(0))) else {
                continue;
            };
            if book.entries.len() >= MAX_ENTRIES {
                break;
            }
            let key = address.email.to_lowercase();
            if book.known.insert(key.clone()) {
                if address.name.is_none() {
                    book.unnamed.insert(key);
                }
                book.entries.push(address);
            }
        }
        book
    }

    /// The text to save: one address per line, most recent first.
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        for address in &self.entries {
            text.push_str(&format(std::slice::from_ref(address)));
            text.push('\n');
        }
        text
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The addresses in order, most recent first.
    pub fn entries(&self) -> &[Address] {
        &self.entries
    }

    /// Addresses seen in a list: the ones not known yet go in front, in the
    /// order they came; a known one stays where it is, and gets the name it
    /// was missing.
    pub fn observe_seen<'a>(&mut self, addresses: impl IntoIterator<Item = &'a Address>) {
        let mut fresh: Vec<Address> = Vec::new();
        for address in addresses {
            let Some(address) = cleaned(address) else {
                continue;
            };
            let key = address.email.to_lowercase();
            if self.known.contains(&key) {
                self.give_name(&key, &address);
            } else if self.known.insert(key.clone()) {
                if address.name.is_none() {
                    self.unnamed.insert(key);
                }
                fresh.push(address);
            }
        }
        self.put_in_front(fresh);
    }

    /// Addresses mail was sent to: each goes to the front, in the order it
    /// came, with the name it had or the one the book already knew. Mail sent
    /// to the addresses at the front already, in that order, changes nothing.
    pub fn observe_sent<'a>(&mut self, addresses: impl IntoIterator<Item = &'a Address>) {
        let mut front: Vec<Address> = Vec::new();
        for address in addresses {
            let Some(mut address) = cleaned(address) else {
                continue;
            };
            if front.iter().any(|done| done.email.eq_ignore_ascii_case(&address.email)) {
                continue;
            }
            if address.name.is_none() {
                address.name = self
                    .entries
                    .iter()
                    .find(|entry| entry.email.eq_ignore_ascii_case(&address.email))
                    .and_then(|entry| entry.name.clone());
            }
            front.push(address);
        }
        let in_place = front.len() <= self.entries.len() && front.iter().zip(&self.entries).all(|(new, old)| new == old);
        if in_place {
            return;
        }
        self.entries.retain(|entry| !front.iter().any(|new| new.email.eq_ignore_ascii_case(&entry.email)));
        for address in &front {
            let key = address.email.to_lowercase();
            self.known.insert(key.clone());
            if address.name.is_none() {
                self.unnamed.insert(key);
            } else {
                self.unnamed.remove(&key);
            }
        }
        self.put_in_front(front);
    }

    fn put_in_front(&mut self, mut addresses: Vec<Address>) {
        if addresses.is_empty() {
            return;
        }
        self.changed = true;
        addresses.append(&mut self.entries);
        self.entries = addresses;
        if self.entries.len() > MAX_ENTRIES {
            for gone in self.entries.drain(MAX_ENTRIES..) {
                let key = gone.email.to_lowercase();
                self.known.remove(&key);
                self.unnamed.remove(&key);
            }
        }
    }

    /// Gives the entry for `key` the name `seen` carries, when the entry has
    /// none.
    fn give_name(&mut self, key: &str, seen: &Address) {
        if seen.name.is_none() || !self.unnamed.contains(key) {
            return;
        }
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.email.eq_ignore_ascii_case(&seen.email)) {
            entry.name.clone_from(&seen.name);
            self.unnamed.remove(key);
            self.changed = true;
        }
    }

    /// Up to `limit` addresses whose name or address has a word that starts
    /// with what was typed, most recent first, capitals ignored. Addresses
    /// in `skip` (already in the field) are left out.
    pub fn matches(&self, typed: &str, limit: usize, skip: &[String]) -> Vec<Address> {
        let needle = typed.trim().to_lowercase();
        if needle.is_empty() {
            return Vec::new();
        }
        self.entries
            .iter()
            .filter(|entry| !skip.iter().any(|done| done.eq_ignore_ascii_case(&entry.email)))
            .filter(|entry| starts_a_word(entry, &needle))
            .take(limit)
            .cloned()
            .collect()
    }

    /// The text to save when something changed since the last time this was
    /// asked, else `None`.
    pub fn take_text_if_changed(&mut self) -> Option<String> {
        std::mem::take(&mut self.changed).then(|| self.to_text())
    }

    /// A save that failed: the book counts as changed again.
    pub fn mark_unsaved(&mut self) {
        self.changed = true;
    }
}

/// Whether a word of the name or the address starts with `needle`, which is
/// already lowercase. A word starts at the front or after anything that is
/// not a letter or digit, so `ch` finds `Maya Chen` and `acme` finds
/// `maya@acme.dev`.
fn starts_a_word(address: &Address, needle: &str) -> bool {
    let name = address.name.as_deref().unwrap_or("");
    let text = format!("{} {}", name.to_lowercase(), address.email.to_lowercase());
    let mut previous: Option<char> = None;
    for (at, ch) in text.char_indices() {
        if previous.is_none_or(|before| !before.is_alphanumeric()) && text[at..].starts_with(needle) {
            return true;
        }
        previous = Some(ch);
    }
    false
}

/// The address as the book keeps it: a name with no control codes and not
/// absurdly long, and an address a header can carry; `None` for one that is
/// not.
fn cleaned(address: &Address) -> Option<Address> {
    let name = address
        .name
        .as_deref()
        .map(|name| name.chars().filter(|ch| !ch.is_control()).collect::<String>().trim().to_string())
        .filter(|name| !name.is_empty() && name.chars().count() <= MAX_NAME_CHARS);
    let candidate = Address { name, email: address.email.trim().to_string() };
    let mut parsed = parse(&format(std::slice::from_ref(&candidate))).ok()?;
    let found = (parsed.len() == 1).then(|| parsed.remove(0))?;
    (found.email == candidate.email).then_some(found)
}

#[cfg(test)]
include!("address_book.tests.rs");
