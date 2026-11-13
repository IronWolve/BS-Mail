//! The screen with its mail source.
//!
//! Every way of showing it — a terminal, a window, a picture file — starts
//! from here, so all three show the same thing.

use std::path::PathBuf;

use engine::MockBackend;
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
use crate::paths::installed_fixture;
use crate::pump::pump;

pub struct Scene {
    pub view: AppView,
    pub backend: Box<dyn MailBackend>,
    /// Messages the mock holds; zero for a real account.
    pub messages: usize,
    /// Where the mail comes from, for the stats block.
    pub source: PathBuf,
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
}

/// Builds the screen and its mail source.
pub fn build(options: &Options, capabilities: Capabilities) -> Result<Scene, AppError> {
    let source = mock_source(options)?;
    let theme = default_theme().map_err(|error| AppError::Theme(error.to_string()))?;
    let base = base_colors(&theme);
    let mut view = AppView::new(
        identity(),
        theme,
        Config::default(),
        IconSet::new(options.icons),
    );
    view.set_capabilities(capabilities);
    view.set_now(source.backend.now());
    Ok(Scene {
        view,
        messages: source.backend.fixture().messages.len(),
        fixed_now: Some(source.backend.now()),
        backend: Box::new(source.backend),
        source: source.path,
        base,
    })
}

struct MockSource {
    backend: MockBackend,
    path: PathBuf,
}

/// The mock mail: the file named on the command line, else the one beside
/// the program.
fn mock_source(options: &Options) -> Result<MockSource, AppError> {
    let path = match &options.mock {
        Some(path) => path.clone(),
        None => installed_fixture().ok_or(AppError::NoFixture)?,
    };
    let backend = MockBackend::from_file(&path)
        .map_err(|error| AppError::Fixture(path.clone(), error.to_string()))?;
    Ok(MockSource { backend, path })
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
