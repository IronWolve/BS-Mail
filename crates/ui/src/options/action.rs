//! What the Options screen asks the program to do. The screen never touches a
//! file or a server: it queues these, and the program takes them once.

use model::Secret;
use model::config::IconMode;

use super::layout_choices::LayoutChange;

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
    /// A setting of the main screen's layout: where the reader sits, the
    /// spacing, the columns and the panels. The screen has already switched.
    SetLayout(LayoutChange),
    /// Read the settings files again.
    ReloadSettings,
    /// Protect the saved passwords with this new passphrase (it was typed
    /// twice and passed the rule), in place of a key file.
    UsePassphrase(Secret),
    /// Use this new passphrase in place of the one in use.
    ChangePassphrase(Secret),
    /// Protect the saved passwords with a key file again (confirmed).
    UseKeyFile,
    /// Keep the key of the saved passwords in the system keychain (confirmed).
    UseKeychain,
    /// Set the vault that cannot be opened aside (kept, never deleted) and
    /// start a new empty one that a key file protects (confirmed).
    StartVaultOver,
    /// Look whether this computer has a keychain the program can use, and say
    /// so with `set_keychain`.
    CheckKeychain,
}
