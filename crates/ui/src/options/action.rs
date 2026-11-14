//! What the Options screen asks the program to do. The screen never touches a
//! file or a server: it queues these, and the program takes them once.

use model::config::IconMode;

/// One request from the Options screen.
#[derive(Debug, Clone, PartialEq)]
pub enum OptionsAction {
    /// Open the account form for a new account.
    AddAccount,
    /// Open the account form filled with this account (by id).
    EditAccount(String),
    /// Delete this account (by id). Only sent after the user confirmed.
    RemoveAccount(String),
    /// Use the theme with this name.
    SetTheme(String),
    /// Use this icon set; the screen has already switched to it.
    SetIcons(IconMode),
    /// Text size in points, within the allowed range.
    SetFontSize(u8),
    /// Row height multiplier, within the allowed range.
    SetLineSpacing(f32),
    /// Read the settings files again.
    ReloadSettings,
}
