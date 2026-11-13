//! The owner's preferred sizes: what `state.toml` stores and what a drag
//! changes. They are only preferences; `clamp.rs` fits them to the window.

use model::config::ListWidths;

/// Sidebar width as the column of the sidebar/main divider (the left border
/// is column 0, so the sidebar's text columns are 2 up to 27).
pub const DEFAULT_SIDEBAR_WIDTH: u16 = 29;
/// List share of the main area, in permille of its height (or width when
/// the reader is on the right).
pub const DEFAULT_LIST_SHARE: u16 = 380;

/// Preferred sizes with the defaults of the reference screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutPrefs {
    /// Column of the divider between sidebar and main area.
    pub sidebar_width: u16,
    /// List share in permille, 0 to 1000.
    pub list_share: u16,
    /// Rows for the Accounts section; 0 fits its content.
    pub accounts_rows: u16,
    /// Rows for the Status section; 0 fits its content.
    pub status_rows: u16,
    /// Width in cells of each fixed list column.
    pub widths: ListWidths,
}

impl Default for LayoutPrefs {
    fn default() -> Self {
        Self {
            sidebar_width: DEFAULT_SIDEBAR_WIDTH,
            list_share: DEFAULT_LIST_SHARE,
            accounts_rows: 0,
            status_rows: 0,
            widths: ListWidths::default(),
        }
    }
}

#[cfg(test)]
include!("prefs.tests.rs");
