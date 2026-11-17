//! The rows of whichever page is showing.

use super::rows::{Row, controls_of};
use super::state::{OptionsState, Page};
use super::view::OptionsView;
use super::{about_page, accounts_page, appearance_page, data_page};

/// The rows of the current page for a page `width` cells wide.
pub fn page_rows(view: &OptionsView<'_>, state: &OptionsState, width: u16) -> Vec<Row> {
    match state.page {
        Page::Accounts => accounts_page::rows(view, state, width),
        Page::Appearance => appearance_page::rows(view),
        Page::Data => data_page::rows_for(view.paths, view.protection, view.keychain, view.vault_trouble, width),
        Page::About => about_page::rows(view.identity, width),
    }
}

/// The controls of the current page, in Tab order. They do not depend on the
/// width, so none is needed.
pub fn page_controls(view: &OptionsView<'_>, state: &OptionsState) -> Vec<super::control::Control> {
    controls_of(&page_rows(view, state, super::layout::MAX_PAGE_CELLS))
}

#[cfg(test)]
include!("pages.tests.rs");
