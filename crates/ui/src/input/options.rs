//! The Options screen as part of the view: opening it, its input, and how it
//! and the program talk to each other.
//!
//! The screen never touches a file or a server. A press that needs the
//! program queues an `OptionsAction`, taken once; the program answers through
//! the view's setters. While the screen is open no key and no click reaches
//! the mail beneath it, and the account form, when it is open, sits above.

use surface::{ButtonState, InputEvent, LogicalKey, NamedKey, PointerButton, PointerShape};

use crate::app_view::AppView;
use crate::options::{KeychainState, OptionsAction, OptionsState, Page, Protection};
use crate::options::input::{Nav, Outcome, handle_key, handle_text};
use crate::options::layout::OptionsRects;
use crate::options::mouse::{click, is_clickable, wheel};
use crate::options::pages::{page_controls, page_rows};

impl AppView {
    /// Shows the Options screen, on the page of accounts, with the
    /// highlight on the account whose mail is showing.
    pub fn open_options(&mut self) {
        let mut state = OptionsState::new();
        let shown = self.selection.account.as_ref().map(|id| id.0.as_str());
        let at = self.options_host.accounts.as_ref().zip(shown).and_then(|(list, shown)| list.iter().position(|a| a.id == shown));
        state.cursor = at.unwrap_or(0);
        self.options_host.line = None;
        self.options = Some(state);
    }

    /// Whether the Options screen is open (the account form may be above it).
    pub fn is_in_options(&self) -> bool {
        self.options.is_some()
    }

    /// What the Options screen asked the program to do, once each, in order.
    pub fn take_options_actions(&mut self) -> Vec<OptionsAction> {
        std::mem::take(&mut self.options_host.actions)
    }

    pub(crate) fn queue_options_action(&mut self, action: OptionsAction) {
        self.options_host.actions.push(action);
    }

    /// Input while the screen is open. Returns whether the screen changed.
    /// On a grid too small to draw the screen only Escape is used, so no
    /// control the person cannot see (the removal question among them) can be
    /// pressed.
    pub(crate) fn options_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        let rects = Self::options_rects(grid);
        if rects.is_none() && !is_escape(event) {
            return false;
        }
        let Some(mut state) = self.options.take() else {
            return false;
        };
        let typed = is_user_input(event);
        let page = state.page;
        let had_line = typed && self.options_host.line.take().is_some();
        // The mail may have changed since the last input (an account list
        // from the backend): start from something that exists.
        self.settle_options(&mut state, rects.as_ref(), false);
        let outcome = self.options_outcome(&mut state, event, rects.as_ref());
        if outcome == Outcome::Close {
            return true;
        }
        if let Outcome::Press(control, part) = outcome {
            self.options_press(&mut state, control, part);
        }
        let reveal = !matches!(event, InputEvent::Wheel { .. });
        self.settle_options(&mut state, rects.as_ref(), reveal);
        if state.page == Page::Data && page != Page::Data {
            self.ask_for_the_keychain();
        }
        self.options = Some(state);
        had_line || outcome != Outcome::Ignored
    }

    /// The Data page has just been opened: when the program has not found
    /// out yet whether this computer has a keychain (or found none the last
    /// time), asks it to look, once. The page says "checking..." meanwhile.
    fn ask_for_the_keychain(&mut self) {
        let host = &mut self.options_host;
        let worth_asking = host.protection.is_some_and(|now| now != Protection::Keychain);
        if worth_asking && matches!(host.keychain, KeychainState::Unknown | KeychainState::Unavailable) {
            host.keychain = KeychainState::Checking;
            host.actions.push(OptionsAction::CheckKeychain);
        }
    }

    fn options_outcome(&self, state: &mut OptionsState, event: &InputEvent, rects: Option<&OptionsRects>) -> Outcome {
        let view = self.options_view();
        let controls = page_controls(&view, state);
        let nav = Nav { controls: &controls, accounts: self.options_host.account_count() };
        match event {
            InputEvent::Key(key) => handle_key(state, key, &nav),
            InputEvent::Text(text) => handle_text(state, text, &nav),
            InputEvent::Button { button: PointerButton::Left, state: ButtonState::Pressed, pos, .. } => {
                let Some(rects) = rects else {
                    return Outcome::Ignored;
                };
                let rows = page_rows(&view, state, rects.page.width);
                click(state, rects, &rows, pos.col, pos.row)
            }
            InputEvent::Wheel { rows: notches, .. } => {
                let Some(rects) = rects else {
                    return Outcome::Ignored;
                };
                let total = page_rows(&view, state, rects.page.width).len();
                wheel(state, rects, total, *notches)
            }
            _ => Outcome::Ignored,
        }
    }

    /// The pointer over the Options screen: a hand over what a click does.
    pub(crate) fn options_pointer_shape(&self, col: u16, row: u16, grid: (u16, u16)) -> PointerShape {
        let (Some(state), Some(rects)) = (&self.options, Self::options_rects(grid)) else {
            return PointerShape::Arrow;
        };
        let rows = page_rows(&self.options_view(), state, rects.page.width);
        if is_clickable(state, &rects, &rows, col, row) { PointerShape::Hand } else { PointerShape::Arrow }
    }
}

/// Escape with no modifier: the one key that is safe when nothing is drawn.
fn is_escape(event: &InputEvent) -> bool {
    let InputEvent::Key(key) = event else {
        return false;
    };
    let held = key.modifiers;
    !held.ctrl && !held.alt && !held.meta && key.key == LogicalKey::Named(NamedKey::Escape)
}

/// A key, text or a button press: the inputs that clear the old line.
fn is_user_input(event: &InputEvent) -> bool {
    matches!(event, InputEvent::Key(_) | InputEvent::Text(_) | InputEvent::Button { state: ButtonState::Pressed, .. })
}

#[cfg(test)]
include!("options.tests.rs");
