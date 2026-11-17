//! The one event sink a run gives its host: the start-up prompt while the
//! vault is locked, then the screen. The screen is built only once the vault is
//! open (that is what reads and starts the accounts), by a function the run
//! hands in; a run with no passphrase builds it at once and never shows the
//! prompt. A window therefore opens once, and a terminal stays in its own
//! screen, from the prompt to the mail.

use std::path::PathBuf;

use surface::{EventSink, Flow, InputEvent, Presenter};

use super::gate::{Gate, GateStep};
use crate::data_lock::DataLock;
use crate::errors::AppError;
use crate::screen::Screen;

/// What a run keeps from building the screen, for the numbers printed at the end.
#[derive(Debug, Clone, Default)]
pub struct Facts {
    pub messages: usize,
    pub source: PathBuf,
    pub label: &'static str,
    pub accounts: Option<usize>,
    pub effects: usize,
}

/// The screen and what it was built from.
pub struct Built {
    pub screen: Screen,
    pub facts: Facts,
    /// The lock on the data folder, kept until the run ends.
    pub lock: Option<DataLock>,
}

/// Builds the screen, holding the data folder's lock when the prompt took it,
/// for a grid of the given size.
pub type Builder = Box<dyn FnOnce(Option<DataLock>, (u16, u16)) -> Result<Built, AppError>>;

enum Stage {
    Asking(Gate, Option<Builder>),
    Running(Box<Built>),
    /// Ended without a screen: the person quit at the prompt, or building failed.
    Over,
}

pub struct Gated {
    stage: Stage,
    grid: (u16, u16),
    failure: Option<AppError>,
}

impl Gated {
    /// With no `gate` the screen is built now and the run starts on it; with
    /// one, `build` waits until the vault is open.
    pub fn new(gate: Option<Gate>, grid: (u16, u16), build: Builder) -> Result<Self, AppError> {
        let stage = match gate {
            None => Stage::Running(Box::new(build(None, grid)?)),
            Some(gate) => Stage::Asking(gate, Some(build)),
        };
        Ok(Self { stage, grid, failure: None })
    }

    /// The screen and what it was built from, after the run; the error that
    /// stopped the build instead; `None` when the person quit at the prompt.
    pub fn finish(self) -> Result<Option<Built>, AppError> {
        match (self.stage, self.failure) {
            (_, Some(error)) => Err(error),
            (Stage::Running(built), None) => Ok(Some(*built)),
            _ => Ok(None),
        }
    }

    /// The vault is open: build the screen and go on as it.
    fn open(&mut self, host: &mut dyn Presenter) -> Flow {
        let Stage::Asking(gate, build) = std::mem::replace(&mut self.stage, Stage::Over) else {
            return Flow::Quit;
        };
        let Some(build) = build else {
            return Flow::Quit;
        };
        match build(gate.into_lock(), self.grid) {
            Ok(built) => {
                self.stage = Stage::Running(Box::new(built));
                host.request_redraw();
                Flow::Continue
            }
            Err(error) => {
                self.failure = Some(error);
                Flow::Quit
            }
        }
    }
}

impl EventSink for Gated {
    fn on_input(&mut self, event: InputEvent, host: &mut dyn Presenter) -> Flow {
        if let InputEvent::Resized { cols, rows } = &event {
            self.grid = (*cols, *rows);
        }
        let grid = self.grid;
        match &mut self.stage {
            Stage::Running(built) => built.screen.on_input(event, host),
            Stage::Over => Flow::Quit,
            Stage::Asking(gate, _) => match gate.input(&event, host, grid) {
                GateStep::Stay => Flow::Continue,
                GateStep::Open => self.open(host),
                GateStep::Quit => {
                    self.stage = Stage::Over;
                    Flow::Quit
                }
            },
        }
    }

    fn on_wake(&mut self, host: &mut dyn Presenter) -> Flow {
        match &mut self.stage {
            Stage::Running(built) => built.screen.on_wake(host),
            Stage::Over => Flow::Quit,
            Stage::Asking(..) => Flow::Continue,
        }
    }

    fn on_redraw(&mut self, host: &mut dyn Presenter) {
        let grid = self.grid;
        match &mut self.stage {
            Stage::Running(built) => built.screen.on_redraw(host),
            Stage::Asking(gate, _) => gate.redraw(host, grid),
            Stage::Over => {}
        }
    }
}

#[cfg(test)]
include!("gated.tests.rs");
