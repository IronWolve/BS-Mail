//! The screen with the mock data already in it.
//!
//! Every way of showing it — a terminal, a window, a picture file — starts
//! from here, so all three show the same thing.

use std::path::PathBuf;

use engine::MockBackend;
use model::config::Config;
use surface::Capabilities;
use ui::AppView;
use ui::icons::IconSet;
use ui::theme::roles;
use ui::theme::{Theme, default_theme};

use crate::cli::Options;
use crate::errors::AppError;
use crate::identity::identity;
use crate::paths::installed_fixture;

pub struct Scene {
    pub view: AppView,
    pub messages: usize,
    pub fixture: PathBuf,
    /// What the theme means by plain text on the plain background.
    pub base: (host::Rgb, host::Rgb),
}

/// Loads the mock data and the theme and fills the screen with them.
pub fn build(options: &Options, capabilities: Capabilities) -> Result<Scene, AppError> {
    let fixture = match &options.mock {
        Some(path) => path.clone(),
        None => installed_fixture().ok_or(AppError::NoFixture)?,
    };
    let backend = MockBackend::from_file(&fixture)
        .map_err(|error| AppError::Fixture(fixture.clone(), error.to_string()))?;
    let theme = default_theme().map_err(|error| AppError::Theme(error.to_string()))?;
    let base = base_colors(&theme);
    let mut view = AppView::new(
        identity(),
        theme,
        Config::default(),
        IconSet::new(options.icons),
    );
    view.set_capabilities(capabilities);
    view.set_now(backend.now());
    for event in backend.startup_events() {
        view.apply_event(event);
    }
    Ok(Scene {
        view,
        messages: backend.fixture().messages.len(),
        fixture,
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
