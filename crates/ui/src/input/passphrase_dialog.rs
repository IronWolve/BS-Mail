//! The passphrase dialog as part of the screen: opening it from the Options
//! screen, its input, and how what it decided goes to the program. While it
//! is open no key and no click reaches anything beneath it. The dialog is
//! dropped (its typed text overwritten) as soon as it has decided.

use surface::InputEvent;

use crate::app_view::AppView;
use crate::options::OptionsAction;
use crate::passphrase::{DialogKind, DialogStep, PassphraseDialog};

impl AppView {
    /// Opens the dialog of `kind` over the Options screen.
    pub(crate) fn open_passphrase_dialog(&mut self, kind: DialogKind) {
        self.passphrase_dialog = Some(PassphraseDialog::new(kind));
    }

    /// Whether the passphrase dialog is open.
    pub fn is_asking_passphrase(&self) -> bool {
        self.passphrase_dialog.is_some()
    }

    /// Input while the dialog is open. Returns whether the screen changed.
    pub(crate) fn passphrase_dialog_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        let Some(dialog) = self.passphrase_dialog.as_mut() else {
            return false;
        };
        let step = dialog.handle_input(event, grid);
        let action = match step {
            DialogStep::Nothing => return false,
            DialogStep::Redraw => return true,
            DialogStep::Cancel => None,
            DialogStep::UsePassphrase(secret) => Some(OptionsAction::UsePassphrase(secret)),
            DialogStep::ChangePassphrase(secret) => Some(OptionsAction::ChangePassphrase(secret)),
            DialogStep::UseKeyFile => Some(OptionsAction::UseKeyFile),
        };
        // It has decided: the typed text goes now, not when the view next changes.
        self.passphrase_dialog = None;
        if let Some(action) = action {
            self.options_host.line = None;
            self.queue_options_action(action);
        }
        true
    }
}

#[cfg(test)]
include!("passphrase_dialog.tests.rs");
