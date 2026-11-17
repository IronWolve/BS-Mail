//! The prompt shown at the start of a run whose vault is protected by a
//! passphrase, before any account starts. It is a screen of its own, not part
//! of the mail view: the program builds it from the theme and settings, feeds
//! it input and acts on what it answers (`UnlockStep`). The passphrase is
//! typed into a buffer that is overwritten when the view goes away, and only
//! dots are drawn. "Forgot the passphrase" explains what it costs, then asks a
//! second time before the program sets the vault aside.

use std::fmt;

use model::config::Config;
use surface::Capabilities;

use super::frame::Focus;
use super::layout::{Block, ButtonId, FieldId, Prompt, Tone};
use super::line::SecretLine;
use crate::icons::{IconKind, IconSet};
use crate::theme::Theme;

const INTRO: &str = "The saved passwords of your accounts are protected by a passphrase. Type it to open them.";
const FORGOT: &str = "A forgotten passphrase cannot be recovered. The saved passwords and sign-ins are set aside \
    (the old file is kept as secrets/vault.unreadable), and you enter each account's password again, or sign in \
    again. Your mail is not lost: it stays on the server.";
const LOST_MISSING: &str = "The saved passwords should be protected by a passphrase, but their vault file is not there, \
    so no passphrase can open it. If you carry on, a new vault with a key file is started and you enter each \
    password again, or sign in again. Your mail is not lost: it stays on the server. To try a copy, quit and put it \
    back.";
const LOST_DAMAGED: &str = "The saved passwords should be protected by a passphrase, but their vault file is damaged. If \
    you carry on, it is set aside as secrets/vault.unreadable (or the next free name), a new vault with a key file is \
    started and you enter each password again, or sign in again. Your mail is not lost. To try a copy, quit and put \
    it back.";
pub const WRONG: &str = "That passphrase does not open the vault. Try again.";
pub const EMPTY: &str = "Type the passphrase first.";

/// Which question is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnlockStage {
    /// Asking for the passphrase.
    Entering,
    /// "Forgot the passphrase" was chosen: the cost is stated, and the second
    /// confirmation is awaited.
    ConfirmForget,
    /// The vault of a passphrase folder is missing or damaged, so there is no
    /// passphrase to ask for: what carrying on costs is stated, and the safe
    /// answer (Quit, which changes nothing) has the keyboard.
    Lost,
}

/// What the program is to do after an input.
#[derive(Debug, PartialEq, Eq)]
pub enum UnlockStep {
    /// Nothing changed.
    Nothing,
    /// The screen changed; draw it again.
    Redraw,
    /// Try this passphrase. The program answers with `wrong_passphrase`,
    /// `failed`, or goes on.
    Unlock(model::Secret),
    /// The second confirmation was given: set the vault aside (or, for a
    /// vault that is missing or damaged, carry on without it).
    Forget,
    /// Leave the program.
    Quit,
}

pub struct UnlockView {
    pub(super) theme: Theme,
    pub(super) icons: IconSet,
    pub(super) config: Config,
    pub(super) line: SecretLine,
    pub(super) stage: UnlockStage,
    pub(super) focus: Focus,
    pub(super) problem: Option<String>,
    /// The letter of a command key just handled, whose text is still to come.
    pub(super) paired: Option<char>,
    /// For the `Lost` stage: the vault file is not there at all (as against
    /// being damaged).
    pub(super) missing: bool,
}

impl fmt::Debug for UnlockView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnlockView").field("stage", &self.stage).finish_non_exhaustive()
    }
}

impl UnlockView {
    pub fn new(theme: Theme, config: Config, kind: IconKind) -> Self {
        let icons = IconSet::new(kind);
        Self {
            theme,
            icons,
            config,
            line: SecretLine::new(),
            stage: UnlockStage::Entering,
            focus: Focus { field: Some(FieldId::First), button: None },
            problem: None,
            paired: None,
            missing: false,
        }
    }

    /// The screen for a passphrase folder whose vault is missing (`missing`) or
    /// damaged: there is nothing a passphrase could open, so none is asked for.
    /// Carrying on (never the default) sets what is there aside and starts a
    /// key-file vault; Quit changes nothing.
    pub fn lost(theme: Theme, config: Config, kind: IconKind, missing: bool) -> Self {
        let mut view = Self::new(theme, config, kind);
        view.stage = UnlockStage::Lost;
        view.focus = Focus { field: None, button: Some(ButtonId::Quit) };
        view.missing = missing;
        view
    }

    /// What the host showing the prompt can do, for the colours.
    pub fn set_capabilities(&mut self, capabilities: Capabilities) {
        if let Some(shown) = self.theme.for_depth(capabilities.color_depth) {
            self.theme = shown;
        }
    }

    pub fn stage(&self) -> UnlockStage {
        self.stage
    }

    /// How many characters are typed (the screen shows this many dots).
    pub fn typed_chars(&self) -> usize {
        self.line.chars()
    }

    /// The problem line, if one is showing.
    pub fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    /// The passphrase did not open the vault: it is cleared, the problem is
    /// said and the person can try again at once.
    pub fn wrong_passphrase(&mut self) {
        self.line.clear();
        self.stage = UnlockStage::Entering;
        self.focus = Focus { field: Some(FieldId::First), button: None };
        self.problem = Some(WRONG.to_string());
    }

    /// Something else went wrong (the vault could not be read, or could not
    /// be set aside); the words come from the program and carry no secret.
    pub fn failed(&mut self, text: &str) {
        if self.stage == UnlockStage::Lost {
            // No passphrase to go back to: the safe answer has the keyboard again.
            self.focus = Focus { field: None, button: Some(ButtonId::Quit) };
        } else {
            self.stage = UnlockStage::Entering;
            self.focus = Focus { field: Some(FieldId::First), button: None };
        }
        self.problem = Some(text.to_string());
    }

    /// What is on the box now.
    pub(crate) fn prompt(&self) -> Prompt {
        match self.stage {
            UnlockStage::Entering => Prompt {
                title: " Passphrase needed ",
                hint: " Enter unlock   Esc quit ",
                blocks: vec![
                    Block::Blank,
                    Block::Text(INTRO.to_string(), Tone::Plain),
                    Block::Blank,
                    Block::Field(FieldId::First, "Passphrase"),
                    Block::Notice(3, self.problem.clone().map(|text| (text, Tone::Problem))),
                    Block::Blank,
                    Block::Buttons(vec![
                        (ButtonId::Unlock, "Unlock"),
                        (ButtonId::Forgot, "Forgot the passphrase"),
                        (ButtonId::Quit, "Quit"),
                    ]),
                ],
            },
            UnlockStage::Lost => Prompt {
                title: if self.missing { " Vault missing " } else { " Vault damaged " },
                hint: " y carry on   Esc quit ",
                blocks: vec![
                    Block::Blank,
                    Block::Text((if self.missing { LOST_MISSING } else { LOST_DAMAGED }).to_string(), Tone::Plain),
                    // One row that is the gap above the buttons until a failure needs it.
                    Block::Notice(1, self.problem.clone().map(|text| (text, Tone::Problem))),
                    Block::Buttons(vec![
                        (ButtonId::SetAside, if self.missing { "Start a new vault" } else { "Set the passwords aside" }),
                        (ButtonId::Quit, "Quit"),
                    ]),
                ],
            },
            UnlockStage::ConfirmForget => Prompt {
                title: " Forgot the passphrase ",
                hint: " y set aside   Esc go back ",
                blocks: vec![
                    Block::Blank,
                    Block::Text(FORGOT.to_string(), Tone::Plain),
                    Block::Blank,
                    Block::Buttons(vec![(ButtonId::SetAside, "Set the passwords aside"), (ButtonId::Back, "Go back")]),
                ],
            },
        }
    }
}

#[cfg(test)]
include!("unlock.tests.rs");
