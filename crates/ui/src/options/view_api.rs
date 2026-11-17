//! What the program tells the view about accounts, places and settings, and
//! how the view takes a new theme, icon set or configuration.

use model::config::{Config, IconMode};

use super::host::InfoLine;
use super::layout::{OptionsRects, layout, max_scroll};
use super::protection::{KeychainState, Protection};
use super::pages::{page_controls, page_rows};
use super::rows::row_of;
use super::state::{OptionsState, Stop};
use super::summary::{AccountSummary, OptionsPaths};
use super::view::OptionsView;
use crate::app_view::AppView;
use crate::icons::{IconKind, IconSet};
use crate::theme::Theme;
use crate::view_state::first_row_showing;

impl AppView {
    /// The accounts the program has set up, in the order the Accounts page
    /// lists them. An empty list means no account is set up, and the list
    /// pane says how to add one. Until this is called the view does not know
    /// (mock mail), and says nothing.
    pub fn set_accounts(&mut self, accounts: Vec<AccountSummary>) {
        self.options_host.accounts = Some(accounts);
        self.settle_open_options();
    }

    /// The places the Data page names.
    pub fn set_options_paths(&mut self, paths: OptionsPaths) {
        self.options_host.paths = paths;
    }

    /// How the saved passwords are protected now, for the Data page. Told at
    /// start and again after the user changes it.
    pub fn set_protection(&mut self, protection: Protection) {
        self.options_host.protection = Some(protection);
        self.settle_open_options();
    }

    /// How the saved passwords are protected, as the program last said.
    pub fn protection(&self) -> Option<Protection> {
        self.options_host.protection
    }

    /// Why the vault cannot be opened (`Some`, the program's sentence), when
    /// only the explicit "Start the vault over" is the way out; `None` once it
    /// opens. The Data page offers the action only while this is set.
    pub fn set_vault_trouble(&mut self, trouble: Option<String>) {
        self.options_host.vault_trouble = trouble;
        self.settle_open_options();
    }

    /// What the program last said about the vault that cannot be opened.
    pub fn vault_trouble(&self) -> Option<&str> {
        self.options_host.vault_trouble.as_deref()
    }

    /// What the program found out about the system keychain: the Data page
    /// offers it only when it is `Available`.
    pub fn set_keychain(&mut self, state: KeychainState) {
        self.options_host.keychain = state;
        self.settle_open_options();
    }

    /// What the program last said about the system keychain.
    pub fn keychain(&self) -> KeychainState {
        self.options_host.keychain
    }

    /// The themes the Theme choice goes through.
    pub fn set_theme_names(&mut self, names: Vec<String>) {
        self.options_host.theme_names = names;
    }

    /// Switches the drawn theme at once.
    pub fn set_theme(&mut self, theme: Theme) {
        self.config.appearance.theme = theme.name().to_string();
        self.theme = self.at_host_depth(theme);
    }

    /// Switches the drawn icon set at once. Auto is the icon-font set.
    pub fn set_icon_mode(&mut self, mode: IconMode) {
        self.config.appearance.icons = mode;
        self.icons = IconSet::from_mode(mode, IconKind::Nerd);
    }

    /// Takes settings that were read again: the configuration, the theme and
    /// the icon set made from it.
    pub fn apply_settings(&mut self, config: Config, theme: Theme, icons: IconSet) {
        self.keys = crate::keys::KeyMap::new(&config.keys);
        self.take_list_settings(&config.list);
        self.config = config;
        self.theme = self.at_host_depth(theme);
        self.icons = icons;
    }

    /// `theme` as the host showing these frames has to draw it.
    fn at_host_depth(&self, theme: Theme) -> Theme {
        theme.for_depth(self.capabilities().color_depth).unwrap_or(theme)
    }

    /// A line for the user at the bottom of the Options screen.
    pub fn options_info(&mut self, text: &str) {
        self.options_host.line = Some(InfoLine { text: text.to_string(), problem: false });
    }

    /// A problem, in the error colour, on the same line.
    pub fn options_problem(&mut self, text: &str) {
        self.options_host.line = Some(InfoLine { text: text.to_string(), problem: true });
    }

    /// The facts the pages are built from.
    pub(crate) fn options_view(&self) -> OptionsView<'_> {
        OptionsView {
            accounts: self.options_host.accounts.as_deref(),
            paths: &self.options_host.paths,
            identity: &self.identity,
            theme_names: &self.options_host.theme_names,
            config: &self.config,
            selected: self.selection.account.as_ref().map(|id| id.0.as_str()),
            connections: &self.mail.accounts,
            problem: self.mail.error.as_deref(),
            line: self.options_host.line.as_ref(),
            protection: self.options_host.protection,
            keychain: self.options_host.keychain,
            vault_trouble: self.options_host.vault_trouble.as_deref(),
        }
    }

    /// Keeps the open screen consistent after the accounts changed under it.
    fn settle_open_options(&mut self) {
        if let Some(mut state) = self.options.take() {
            self.settle_options(&mut state, None, false);
            self.options = Some(state);
        }
    }

    /// Keeps the highlight on an account, the keyboard on a control that
    /// exists and the scroll inside the page; with `reveal`, also scrolls the
    /// page so the control that has the keyboard shows.
    pub(crate) fn settle_options(&self, state: &mut OptionsState, rects: Option<&OptionsRects>, reveal: bool) {
        let count = self.options_host.account_count();
        state.clamp_cursor(count);
        if state.confirming && count == 0 {
            state.confirming = false;
        }
        let view = self.options_view();
        state.fix(&page_controls(&view, state));
        let Some(rects) = rects else {
            return;
        };
        let rows = page_rows(&view, state, rects.page.width);
        let height = rects.page.height;
        state.scroll = state.scroll.min(max_scroll(rows.len(), height));
        if let (true, Stop::Control(control)) = (reveal, state.stop)
            && let Some(row) = row_of(&rows, control, state.cursor)
        {
            state.scroll = first_row_showing(state.scroll, row, usize::from(height), rows.len());
        }
    }

    /// The rectangles of the Options screen for a grid, when it is big enough.
    pub(crate) fn options_rects(grid: (u16, u16)) -> Option<OptionsRects> {
        layout(crate::setup::layout::area_of(grid))
    }
}

#[cfg(test)]
include!("view_api.tests.rs");
