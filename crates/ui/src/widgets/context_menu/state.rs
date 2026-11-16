//! What an open menu holds: its entries, the one the cursor is on, and where
//! it was opened. Generic over the action an entry stands for, so the widget
//! knows nothing about what the entries do.

/// One line of a menu: what it does, how it reads, and the letter that
/// chooses it. The letter is one of the label's own, shown underlined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuEntry<A> {
    pub action: A,
    pub label: String,
    pub hotkey: char,
}

impl<A> MenuEntry<A> {
    pub fn new(action: A, label: impl Into<String>, hotkey: char) -> Self {
        Self { action, label: label.into(), hotkey: hotkey.to_ascii_lowercase() }
    }

    /// The index in the label of the character the hotkey underlines: its
    /// first occurrence, in either case.
    pub fn hotkey_at(&self) -> Option<usize> {
        self.label.chars().position(|ch| ch.to_ascii_lowercase() == self.hotkey)
    }
}

/// A menu that is open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu<A> {
    /// A line set in the top border, such as how many messages are marked.
    pub title: Option<String>,
    pub entries: Vec<MenuEntry<A>>,
    /// The highlighted entry: the keyboard moves it and so does the pointer.
    pub cursor: usize,
    /// The cell the menu was opened at: the pointer, or the item's corner.
    pub anchor: (u16, u16),
}

impl<A: Copy> Menu<A> {
    pub fn new(anchor: (u16, u16), title: Option<String>, entries: Vec<MenuEntry<A>>) -> Self {
        Self { title, entries, cursor: 0, anchor }
    }

    /// Moves the highlight one entry, wrapping round at either end.
    pub fn step(&mut self, down: bool) {
        let count = self.entries.len();
        if count == 0 {
            return;
        }
        self.cursor = if down { (self.cursor + 1) % count } else { (self.cursor + count - 1) % count };
    }

    pub fn first(&mut self) {
        self.cursor = 0;
    }

    pub fn last(&mut self) {
        self.cursor = self.entries.len().saturating_sub(1);
    }

    /// The entry a typed letter chooses, in either case.
    pub fn by_hotkey(&self, letter: char) -> Option<usize> {
        let letter = letter.to_ascii_lowercase();
        self.entries.iter().position(|entry| entry.hotkey == letter)
    }

    /// The action of the highlighted entry.
    pub fn chosen(&self) -> Option<A> {
        self.entries.get(self.cursor).map(|entry| entry.action)
    }
}

#[cfg(test)]
include!("state.tests.rs");
