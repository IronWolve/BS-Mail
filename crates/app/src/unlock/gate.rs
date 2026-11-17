//! The start-up prompt as seen from the program: it holds the vault and the
//! lock on the data folder, feeds the `ui` prompt its input and acts on what
//! the prompt decides. It is the first half of `Gated`.

use std::sync::Arc;

use engine::vault::{Mode, SharedVault, VaultError};
use surface::{InputEvent, Presenter};
use ui::passphrase::{UnlockStep, UnlockView};

use crate::cli::Options;
use crate::config_io;
use crate::data_lock::{self, DataLock};
use crate::errors::AppError;
use crate::paths::data_dir;
use crate::scene::icon_kind;

use super::target::{data_folder, vault_folder};

/// What the prompt came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateStep {
    /// Still asking.
    Stay,
    /// The vault is open (or was set aside, as asked): go on.
    Open,
    /// Leave the program.
    Quit,
}

pub struct Gate {
    view: UnlockView,
    vault: Arc<SharedVault>,
    /// The lock on the data folder, taken before the first question and handed
    /// on to the build of the screen.
    held: Option<DataLock>,
}

impl Gate {
    /// The prompt this run needs: `Some` only when its vault is protected by a
    /// passphrase that has not been given. A second copy of the program is
    /// turned away here, before it asks for anything.
    pub fn for_run(options: &Options) -> Result<Option<Self>, AppError> {
        let Some(folder) = vault_folder(options) else {
            return Ok(None);
        };
        let vault = SharedVault::for_dir(&folder);
        let state = vault.protection();
        if state.mode != Mode::Passphrase || !state.locked {
            return Ok(None);
        }
        let held = match data_folder(options) {
            Some(data) => data_lock::hold(&data)?,
            None => None,
        };
        let loaded = config_io::load(data_dir(options.data_dir.as_ref()).as_deref())?;
        let icons = options.icons.unwrap_or_else(|| icon_kind(loaded.config.appearance.icons));
        Ok(Some(Self { view: UnlockView::new(loaded.theme, loaded.config, icons), vault, held }))
    }

    /// The lock on the data folder, for the build that follows.
    pub fn into_lock(self) -> Option<DataLock> {
        self.held
    }

    /// The prompt drawn for the host.
    pub fn redraw(&mut self, host: &mut dyn Presenter, grid: (u16, u16)) {
        self.view.set_capabilities(host.capabilities());
        let frame = self.view.render(grid.0, grid.1);
        host.present(&frame);
    }

    /// One input event.
    pub fn input(&mut self, event: &InputEvent, host: &mut dyn Presenter, grid: (u16, u16)) -> GateStep {
        match event {
            InputEvent::CloseRequested => return GateStep::Quit,
            InputEvent::Resized { .. } | InputEvent::Focus { .. } => {
                host.request_redraw();
                return GateStep::Stay;
            }
            InputEvent::PointerMoved { pos, .. } => host.set_pointer_shape(self.view.pointer_shape(pos.col, pos.row, grid)),
            _ => {}
        }
        match self.view.handle_input(event, grid) {
            UnlockStep::Nothing => GateStep::Stay,
            UnlockStep::Redraw => self.stay_and_redraw(host),
            UnlockStep::Quit => GateStep::Quit,
            UnlockStep::Unlock(secret) => match self.vault.unlock(&secret) {
                Ok(()) => GateStep::Open,
                Err(VaultError::WrongPassphrase) => {
                    self.view.wrong_passphrase();
                    self.stay_and_redraw(host)
                }
                Err(other) => {
                    self.view.failed(&other.to_string());
                    self.stay_and_redraw(host)
                }
            },
            UnlockStep::Forget => match self.vault.forget_passphrase() {
                Ok(()) => GateStep::Open,
                Err(error) => {
                    self.view.failed(&format!("The vault could not be set aside: {error}"));
                    self.stay_and_redraw(host)
                }
            },
        }
    }

    fn stay_and_redraw(&mut self, host: &mut dyn Presenter) -> GateStep {
        host.request_redraw();
        GateStep::Stay
    }
}

#[cfg(test)]
include!("gate.tests.rs");
