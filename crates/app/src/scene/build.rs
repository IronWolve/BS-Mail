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
use crate::errors::AppError;
use crate::identity::identity;
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
    let loaded = config_io::load(data.as_deref())?;
    let base = base_colors(&loaded.theme);
    let font = (
        options.points.unwrap_or(loaded.config.font.size),
        options.line_spacing.unwrap_or(loaded.config.font.line_spacing),
    );
    let icons = options.icons.unwrap_or_else(|| icon_kind(loaded.config.appearance.icons));
    let mut view = AppView::new(identity(), loaded.theme, loaded.config, IconSet::new(icons));
    // What is wrong with an account comes before what is wrong with a setting.
    let problems: Vec<String> = scanned.problems.iter().cloned().chain(loaded.problems).collect();
    view.set_config_problem(problems.first().cloned());
    view.set_capabilities(capabilities);
    view.set_options_paths(options_paths(data.as_deref(), &source));
    let mut scene = scene_for(&source, view, base, &scanned)?;
    scene.font = font;
    if let Some(live) = &scene.live {
        scene.view.set_accounts(live.summaries());
    }
    if options.setup || source.starts_empty() {
        scene.view.open_setup();
    }
    Ok(scene)
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
fn icon_kind(mode: model::config::IconMode) -> ui::icons::IconKind {
    use model::config::IconMode;
    use ui::icons::IconKind;
    match mode {
        IconMode::Auto | IconMode::Nerd => IconKind::Nerd,
        IconMode::Unicode => IconKind::Unicode,
        IconMode::Ascii => IconKind::Ascii,
    }
}

/// The theme's plain text and plain background, which is what a cell means
/// when it names no color of its own.
fn base_colors(theme: &Theme) -> Base {
    let base = theme.role(roles::BASE);
    let background = base.bg.or_else(|| theme.background());
    let pixels = |color: model::theme_schema::Rgb| host::Rgb::new(color.r, color.g, color.b);
    (pixels(base.fg), background.map_or(host::Rgb::new(0, 0, 0), pixels))
}

#[cfg(test)]
include!("build.tests.rs");
