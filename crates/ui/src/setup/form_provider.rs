//! The provider row of the account form: opening the pull-down, what choosing
//! a provider fills in, and the keys and clicks of the open list.
//!
//! The row is set to "detect from my address" until the user picks. Picking a
//! provider fills in its servers, ports, security and note at once, and they
//! stay that way when the address changes; any server field the user edits
//! after that is theirs, as it always was.

use surface::NamedKey;

use super::form::SetupForm;
use super::input::Outcome;
use super::layout::SetupRects;
use super::provider_menu::{MenuOutcome, ProviderChoice, ProviderMenu, VISIBLE_ROWS};
use super::providers::{Preset, generic_preset_for, preset_for, preset_named};

impl SetupForm {
    /// The preset in use for `email`: the provider picked from the list, or
    /// the one the address's domain says.
    pub(super) fn preset_in_use(&self, email: &str) -> Option<Preset> {
        match self.provider {
            ProviderChoice::Auto => preset_for(email),
            ProviderChoice::Named(name) => preset_named(name),
            ProviderChoice::Other => generic_preset_for(email),
        }
    }

    /// Opens the list with the cursor on the provider chosen now.
    pub fn open_provider_menu(&mut self) {
        self.notice = None;
        self.info = None;
        self.quit_armed = false;
        self.provider_menu = Some(ProviderMenu::new(self.provider));
    }

    /// Sets the provider and fills in everything that follows from it. This is
    /// the user's own act, so it replaces servers they had typed and brings
    /// back the way of signing in the provider calls for; a client id they
    /// typed stays.
    pub fn choose_provider(&mut self, choice: ProviderChoice) {
        self.provider = choice;
        self.provider_menu = None;
        self.servers_touched = false;
        self.signin_touched = false;
        self.notice = None;
        self.info = None;
        self.apply_provider();
        if let ProviderChoice::Named(name) = choice
            && self.info.is_none()
        {
            self.info = Some(format!("Filled in the settings for {name}: check them below."));
        }
    }

    /// A key that is not text, while the list is open.
    pub(super) fn menu_key(&mut self, key: NamedKey) -> Outcome {
        let Some(menu) = self.provider_menu.as_mut() else {
            return Outcome::Ignored;
        };
        match menu.key(key, usize::from(VISIBLE_ROWS)) {
            MenuOutcome::Stay => {}
            MenuOutcome::Pick(choice) => self.choose_provider(choice),
            MenuOutcome::Cancel => self.provider_menu = None,
        }
        Outcome::Changed
    }

    /// Typed text while the list is open: it finds a row.
    pub(super) fn menu_text(&mut self, text: &str) -> Outcome {
        if let Some(menu) = self.provider_menu.as_mut() {
            menu.type_text(text);
        }
        Outcome::Changed
    }

    /// A click while the list is open: on a row it picks that provider, and
    /// anywhere else it closes the list and does nothing more.
    pub(crate) fn menu_click(&mut self, rects: &SetupRects, col: u16, row: u16) -> Outcome {
        let menu = rects.menu;
        let inside = menu.height > 0 && row >= menu.y && row < menu.bottom() && col >= menu.x && col < menu.right();
        if !inside {
            self.provider_menu = None;
            return Outcome::Changed;
        }
        let picked = self
            .provider_menu
            .as_mut()
            .and_then(|list| list.point_at(usize::from(row - menu.y), usize::from(menu.height)));
        match picked {
            Some(choice) => {
                self.choose_provider(choice);
                Outcome::Changed
            }
            None => Outcome::Ignored,
        }
    }
}

#[cfg(test)]
include!("form_provider.tests.rs");
