//! The provider pull-down on the account form: the list of providers to pick
//! from, and the cursor, scrolling and type-ahead of the list while it is
//! open. Choosing one fills in its servers whatever the address looks like,
//! which is the way in for a mailbox under the owner's own domain.

use surface::NamedKey;

use super::providers::provider_names;

/// Rows of the list that show at once.
pub const VISIBLE_ROWS: u16 = 10;

/// What the provider row is set to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderChoice {
    /// The servers follow the address, where its domain says who the provider is.
    Auto,
    /// A provider from the list.
    Named(&'static str),
    /// None of them: the usual server names for the address, to be changed.
    Other,
}

impl ProviderChoice {
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Detect from my address",
            Self::Named(name) => name,
            Self::Other => "Other: I will type the servers",
        }
    }
}

/// Every choice, in the order the list shows them: the automatic one, the
/// providers by name, and "other".
pub fn choices() -> Vec<ProviderChoice> {
    let mut all = vec![ProviderChoice::Auto];
    all.extend(provider_names().into_iter().map(ProviderChoice::Named));
    all.push(ProviderChoice::Other);
    all
}

/// What a key did to the open list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuOutcome {
    /// The list is still open (it may have moved).
    Stay,
    /// A choice was made; the list closes.
    Pick(ProviderChoice),
    /// The list closes with nothing changed.
    Cancel,
}

/// The open list: where its cursor is and what has been typed to find a row.
#[derive(Debug, Clone)]
pub struct ProviderMenu {
    choices: Vec<ProviderChoice>,
    cursor: usize,
    typed: String,
}

impl ProviderMenu {
    /// A list with the cursor on `current`.
    pub fn new(current: ProviderChoice) -> Self {
        let choices = choices();
        let cursor = choices.iter().position(|choice| *choice == current).unwrap_or(0);
        Self { choices, cursor, typed: String::new() }
    }

    pub fn choices(&self) -> &[ProviderChoice] {
        &self.choices
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// The first row shown when `visible` rows fit: the cursor row sits near
    /// the middle, and the list never scrolls past its ends.
    pub fn top(&self, visible: usize) -> usize {
        if visible == 0 || self.choices.len() <= visible {
            return 0;
        }
        self.cursor.saturating_sub(visible / 2).min(self.choices.len() - visible)
    }

    /// A key that is not text: it moves, picks or closes.
    pub fn key(&mut self, key: NamedKey, visible: usize) -> MenuOutcome {
        let last = self.choices.len() - 1;
        let page = visible.max(1);
        match key {
            NamedKey::Enter => return MenuOutcome::Pick(self.choices[self.cursor]),
            NamedKey::Escape | NamedKey::Tab => return MenuOutcome::Cancel,
            NamedKey::Up => self.cursor = self.cursor.saturating_sub(1),
            NamedKey::Down => self.cursor = (self.cursor + 1).min(last),
            NamedKey::PageUp => self.cursor = self.cursor.saturating_sub(page),
            NamedKey::PageDown => self.cursor = (self.cursor + page).min(last),
            NamedKey::Home => self.cursor = 0,
            NamedKey::End => self.cursor = last,
            _ => return MenuOutcome::Stay,
        }
        self.typed.clear();
        MenuOutcome::Stay
    }

    /// Typed letters: the cursor goes to the first row that begins with what
    /// has been typed since the last key that was not a letter. A letter that
    /// matches nothing starts the search again from itself.
    pub fn type_text(&mut self, text: &str) {
        let letters: String = text.chars().filter(|ch| !ch.is_control()).collect::<String>().to_lowercase();
        if letters.is_empty() {
            return;
        }
        self.typed.push_str(&letters);
        if self.go_to_prefix(&self.typed.clone()) {
            return;
        }
        self.typed = letters;
        self.go_to_prefix(&self.typed.clone());
    }

    fn go_to_prefix(&mut self, prefix: &str) -> bool {
        let found = self.choices.iter().position(|choice| choice.label().to_lowercase().starts_with(prefix));
        if let Some(index) = found {
            self.cursor = index;
        }
        found.is_some()
    }

    /// Puts the cursor on the row at `offset` below the first row shown, when
    /// there is one, and says whether there was.
    pub fn point_at(&mut self, offset: usize, visible: usize) -> Option<ProviderChoice> {
        let index = self.top(visible) + offset;
        let choice = *self.choices.get(index)?;
        self.cursor = index;
        Some(choice)
    }
}

#[cfg(test)]
include!("provider_menu.tests.rs");
