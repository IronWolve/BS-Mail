//! The screen with its mail source.
//!
//! Every way of showing it — a terminal, a window, a picture file — starts
//! from here, so all three show the same thing.

use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use engine::{AccountConfig, MockBackend, RealBackend};
use model::config::Config;
use model::{MailBackend, MailDate};
use surface::Capabilities;
use ui::AppView;
use ui::icons::IconSet;
use ui::theme::roles;
use ui::theme::{Theme, default_theme};

use crate::cli::Options;
use crate::errors::AppError;
use crate::identity::identity;
use crate::pump::pump;
use crate::source::{Source, choose_for_run};

/// How often a one-shot draw checks whether real mail has arrived.
const SETTLE_STEP: Duration = Duration::from_millis(50);
/// How long a one-shot draw waits for real mail before drawing what it has.
pub const SETTLE_LIMIT: Duration = Duration::from_secs(20);

pub struct Scene {
    pub view: AppView,
    pub backend: Box<dyn MailBackend>,
    /// Messages the mock holds; zero for a real account.
    pub messages: usize,
    /// The account file or mock file the mail comes from.
    pub source: PathBuf,
    /// Real mail from an account, not the mock.
    pub real: bool,
    /// What the theme means by plain text on the plain background.
    pub base: (host::Rgb, host::Rgb),
    /// A clock that stands still, so a picture of mock data never changes
    /// with the day it is taken; `None` follows the system clock.
    pub fixed_now: Option<MailDate>,
}

impl Scene {
    /// Takes everything the backend has ready and answers what the screen
    /// then asks for, so the first frame has data in it.
    pub fn prime(&mut self) {
        pump(&mut self.view, self.backend.as_mut());
    }

    /// For a draw that happens once: as `prime`, and for real mail keeps
    /// asking until the first folder and message have arrived, a failure is
    /// showing, or `limit` runs out. A window does not need this; it fills
    /// in as the mail comes.
    pub fn settle(&mut self, limit: Duration) {
        self.prime();
        if !self.real {
            return;
        }
        let deadline = Instant::now() + limit;
        while !self.is_settled() && Instant::now() < deadline {
            thread::sleep(SETTLE_STEP);
            self.prime();
        }
    }

    fn is_settled(&self) -> bool {
        let mail = self.view.mail();
        if mail.error.is_some() {
            return true;
        }
        if mail.page.is_none() {
            return false;
        }
        match self.view.selection().message {
            Some(id) => mail.body.as_ref().is_some_and(|(shown, _)| *shown == id),
            None => true,
        }
    }
}

/// Builds the screen and its mail source.
pub fn build(options: &Options, capabilities: Capabilities) -> Result<Scene, AppError> {
    let source = choose_for_run(options)?;
    let theme = default_theme().map_err(|error| AppError::Theme(error.to_string()))?;
    let base = base_colors(&theme);
    let mut view = AppView::new(
        identity(),
        theme,
        Config::default(),
        IconSet::new(options.icons),
    );
    view.set_capabilities(capabilities);
    let path = source.path().clone();
    match source {
        Source::Mock(_) => mock_scene(view, base, &path),
        Source::Account(_) => account_scene(view, base, &path),
    }
}

fn mock_scene(
    mut view: AppView,
    base: (host::Rgb, host::Rgb),
    path: &Path,
) -> Result<Scene, AppError> {
    let backend = MockBackend::from_file(path)
        .map_err(|error| AppError::Fixture(path.to_path_buf(), error.to_string()))?;
    view.set_now(backend.now());
    Ok(Scene {
        view,
        messages: backend.fixture().messages.len(),
        fixed_now: Some(backend.now()),
        backend: Box::new(backend),
        source: path.to_path_buf(),
        real: false,
        base,
    })
}

fn account_scene(
    view: AppView,
    base: (host::Rgb, host::Rgb),
    path: &Path,
) -> Result<Scene, AppError> {
    let config = AccountConfig::from_file(path)
        .map_err(|error| AppError::Account(path.to_path_buf(), error.to_string()))?;
    Ok(Scene {
        view,
        messages: 0,
        fixed_now: None,
        backend: Box::new(RealBackend::start(config)),
        source: path.to_path_buf(),
        real: true,
        base,
    })
}

/// The theme's plain text and plain background, which is what a cell means
/// when it names no color of its own.
fn base_colors(theme: &Theme) -> (host::Rgb, host::Rgb) {
    let base = theme.role(roles::BASE);
    let background = base.bg.or_else(|| theme.background());
    let pixels = |color: model::theme_schema::Rgb| host::Rgb::new(color.r, color.g, color.b);
    (
        pixels(base.fg),
        background.map_or(host::Rgb::new(0, 0, 0), pixels),
    )
}
