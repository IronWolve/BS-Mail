//! Asking for a passphrase: the prompt at the start of a run whose vault is
//! protected by one, and the dialog on the Options screen that sets, changes
//! or gives up a passphrase. Both draw only dots for what is typed, keep it in
//! a buffer that is overwritten when it goes away, and never show it. This
//! file only declares and re-exports the modules.

pub mod dialog;
pub mod dialog_draw;
pub mod dialog_input;
pub mod frame;
pub mod layout;
pub mod line;
pub mod unlock;
pub mod unlock_draw;
pub mod unlock_input;

pub use dialog::{DialogKind, DialogStep, PassphraseDialog};
pub use line::SecretLine;
pub use unlock::{UnlockStage, UnlockStep, UnlockView};
