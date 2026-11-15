//! Building the screen and its mail source from what was asked for.

use std::path::Path;

use surface::Capabilities;
use ui::icons::IconSet;
use ui::theme::Theme;
use ui::theme::roles;
use ui::{AppView, OptionsPaths};

use crate::accounts::{Place, Scan, scan};
use crate::cli::Options;
use crate::config_io::{self, CONFIG_NAME};
use crate::data_lock::{self, DataLock, Refused};
use crate::errors::AppError;
use crate::identity::identity;
use crate::options_flow::Settings;
use crate::paths::{accounts_dir, data_dir};
use crate::source::{Source, choose, needs_scan};

use super::sources::{self, Base};
use super::state::Scene;

/// Builds the screen and its mail source.
pub fn build(options: &Options, capabilities: Capabilities) -> Result<Scene, AppError> {
    let data = data_dir(options.data_dir.as_ref());
    let root = data.as_deref().map(accounts_dir);
    let scanned = match (&root, needs_scan(options)) {
        (Some(root), true) => scan(root),
        _ => Scan::default(),
    };
    let source = choose(options, root, scanned.found.len())?;
    // Listing the accounts above only reads; from here on the run may start
    // sign-ins and write, so only one copy at a time gets to.
    let lock = hold_data_folder(&source)?;
    let loaded = config_io::load(data.as_deref())?;
    let base = base_colors(&loaded.theme);
    let font = (
        options.points.unwrap_or(loaded.config.font.size),
        options.line_spacing.unwrap_or(loaded.config.font.line_spacing),
    );
    let icons = options.icons.unwrap_or_else(|| icon_kind(loaded.config.appearance.icons));
    engine::set_body_preference(loaded.config.reader.prefer);
    let mut view = AppView::new(identity(), loaded.theme, loaded.config, IconSet::new(icons));
    // What is wrong with an account comes before what is wrong with a setting.
    let note_is_about_settings = scanned.problems.is_empty() && !loaded.problems.is_empty();
    let problems: Vec<String> = scanned.problems.iter().cloned().chain(loaded.problems).collect();
    view.set_config_problem(problems.first().cloned());
    view.set_capabilities(capabilities);
    view.set_options_paths(options_paths(data.as_deref(), &source));
    let mut scene = scene_for(&source, view, base, &scanned)?;
    scene.lock = lock;
    scene.font = font;
    scene.settings = Settings::start(options, data.as_deref(), font, note_is_about_settings);
    scene.view.set_theme_names(scene.settings.theme_names().to_vec());
    if let Some(live) = &scene.live {
        scene.view.set_accounts(live.summaries());
    }
    if options.setup || source.starts_empty() {
        scene.view.open_setup();
    }
    Ok(scene)
}

/// The lock on the folder this run keeps its accounts in; a second copy is
/// turned away with the name of the first.
fn hold_data_folder(source: &Source) -> Result<Option<DataLock>, AppError> {
    let Some(folder) = source.data_folder() else {
        return Ok(None);
    };
    data_lock::take(&folder).map_err(|refused| match refused {
        Refused::Busy { holder } => AppError::Locked(folder, holder),
        Refused::Unlockable { reason } => AppError::Unlockable(folder, reason),
    })
}

fn scene_for(source: &Source, view: AppView, base: Base, scanned: &Scan) -> Result<Scene, AppError> {
    match source {
        Source::Mock(path) => sources::mock(view, base, path),
        Source::Account(file) => sources::single(view, base, file),
        Source::Accounts(root) => Ok(sources::many(view, base, root, scanned)),
        Source::Empty(root) => Ok(sources::empty(view, base, Place::Root(root.clone()), root.clone())),
        Source::Setup(file) => Ok(sources::empty(view, base, Place::File(file.clone()), file.clone())),
    }
}

/// The places the Data page names. A place that is not known is left empty.
fn options_paths(data: Option<&Path>, source: &Source) -> OptionsPaths {
    let show = |path: &Path| path.display().to_string();
    let accounts = match source {
        Source::Account(file) | Source::Setup(file) => {
            show(&Place::File(file.clone()).accounts_folder())
        }
        _ => data.map(|dir| show(&accounts_dir(dir))).unwrap_or_default(),
    };
    OptionsPaths {
        data_dir: data.map(show).unwrap_or_default(),
        config_file: data.map(|dir| show(&dir.join(CONFIG_NAME))).unwrap_or_default(),
        accounts_dir: accounts,
    }
}

/// The icon set a setting asks for. `auto` is the Nerd set: the window
/// carries its own font, and a terminal's first-run check is not built yet.
pub(crate) fn icon_kind(mode: model::config::IconMode) -> ui::icons::IconKind {
    use model::config::IconMode;
    use ui::icons::IconKind;
    match mode {
        IconMode::Auto | IconMode::Nerd => IconKind::Nerd,
        IconMode::Unicode => IconKind::Unicode,
        IconMode::Ascii => IconKind::Ascii,
    }
}

/// The theme's plain text and plain background, which is what a cell means
/// when it names no color of its own. A theme with no background is black.
pub(crate) fn plain_colors(theme: &Theme) -> (model::theme_schema::Rgb, model::theme_schema::Rgb) {
    let base = theme.role(roles::BASE);
    let background = base.bg.or_else(|| theme.background());
    (base.fg, background.unwrap_or(model::theme_schema::Rgb { r: 0, g: 0, b: 0 }))
}

/// The same two colors, as the window's pixels.
fn base_colors(theme: &Theme) -> Base {
    let (text, background) = plain_colors(theme);
    let pixels = |color: model::theme_schema::Rgb| host::Rgb::new(color.r, color.g, color.b);
    (pixels(text), pixels(background))
}

#[cfg(test)]
include!("build.tests.rs");
