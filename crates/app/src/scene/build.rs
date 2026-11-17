//! Building the screen and its mail source from what was asked for.

use std::path::Path;

use model::config::WindowConfig;
use surface::Capabilities;
use ui::icons::{IconKind, IconSet};
use ui::theme::Theme;
use ui::theme::roles;
use ui::options::{KeychainState, Protection};
use ui::{AppView, OptionsPaths};

use crate::accounts::{Migration, Place, Scan, move_secrets_into_vault, recover, scan};
use crate::address_file;
use crate::cli::Options;
use crate::config_io::{self, CONFIG_NAME};
use crate::data_lock::{self, DataLock};
use crate::errors::AppError;
use crate::identity::identity;
use crate::options_flow::Settings;
use crate::paths::{accounts_dir, data_dir};
use crate::source::{Source, choose, needs_scan};
use crate::unlock::open_keychain_vault;

use super::sources::{self, Base};
use super::state::Scene;

/// What a window and the start-up prompt need before any account is read:
/// everything that comes from the settings and nothing from the vault.
pub struct Look {
    pub icons: IconKind,
    /// Text size in points and row height, from the command line or the settings.
    pub font: (u8, f32),
    /// What the settings say about the window's size and start state.
    pub window: WindowConfig,
    /// What the theme means by plain text on the plain background.
    pub base: Base,
}

/// The look of this run, from the settings.
pub fn look(options: &Options) -> Result<Look, AppError> {
    let data = data_dir(options.data_dir.as_ref());
    let loaded = config_io::load(data.as_deref())?;
    Ok(look_of(options, &loaded))
}

fn look_of(options: &Options, loaded: &config_io::Loaded) -> Look {
    Look {
        icons: options.icons.unwrap_or_else(|| icon_kind(loaded.config.appearance.icons)),
        font: (
            options.points.unwrap_or(loaded.config.font.size),
            options.line_spacing.unwrap_or(loaded.config.font.line_spacing),
        ),
        window: loaded.config.window.clone(),
        base: base_colors(&loaded.theme),
    }
}

/// Builds the screen and its mail source.
pub fn build(options: &Options, capabilities: Capabilities) -> Result<Scene, AppError> {
    build_holding(options, capabilities, None)
}

/// As `build`, for a run that already holds the lock on the data folder
/// (the start-up prompt takes it before it asks for anything).
pub fn build_holding(options: &Options, capabilities: Capabilities, held: Option<DataLock>) -> Result<Scene, AppError> {
    // A vault whose key is in the system keychain is opened first of all, with no prompt: what
    // comes below reads the accounts, and an account that needs a secret says why when it cannot.
    let _ = open_keychain_vault(options);
    let data = data_dir(options.data_dir.as_ref());
    let root = data.as_deref().map(accounts_dir);
    let mut scanned = match (&root, needs_scan(options)) {
        (Some(root), true) => scan(root),
        _ => Scan::default(),
    };
    let source = choose(options, root, scanned.found.len())?;
    // Listing the accounts above only reads; from here on the run may start
    // sign-ins and write, so only one copy at a time gets to.
    let lock = match held {
        Some(lock) => Some(lock),
        None => hold_data_folder(&source)?,
    };
    let recovered = put_right_after_a_hard_stop(&source);
    let moved = move_secrets(&source);
    if moved.moved > 0
        && let Source::Accounts(root) = &source
    {
        // The files changed: what the first look found is out of date.
        scanned = scan(root);
    }
    scanned.problems.extend(recovered);
    let loaded = config_io::load(data.as_deref())?;
    let Look { icons, font, window, base, .. } = look_of(options, &loaded);
    engine::set_body_preference(loaded.config.reader.prefer);
    let mut view = AppView::new(identity(), loaded.theme, loaded.config, IconSet::new(icons));
    if let Some(link) = &options.mailto {
        view.start_with_mailto(link);
    }
    // What is wrong with an account comes before what is wrong with a setting.
    let note_is_about_settings = scanned.problems.is_empty() && !loaded.problems.is_empty();
    // What the move into the vault did comes last: it is said once, and what
    // needs the user's attention comes before it.
    let problems: Vec<String> = scanned.problems.iter().cloned().chain(loaded.problems).chain(moved.notes).collect();
    view.set_config_problem(config_io::footer_note(&problems));
    view.set_capabilities(capabilities);
    view.set_options_paths(options_paths(data.as_deref(), &source));
    tell_how_the_vault_is_kept(&mut view, &source);
    let mut scene = scene_for(&source, view, base, &scanned)?;
    scene.lock = lock;
    scene.font = font;
    scene.window = window;
    scene.settings = Settings::start(options, data.as_deref(), font, note_is_about_settings);
    scene.view.set_theme_names(scene.settings.theme_names().to_vec());
    start_with_accounts(&mut scene, data.as_deref());
    // With no account the program starts on the empty screen, which says where
    // to add one; the form opens only when `--setup` asks for it.
    if options.setup {
        scene.view.open_setup();
    }
    Ok(scene)
}

/// What a run with real accounts has that mock mail does not: the account
/// summaries for the Options screen and the addresses seen so far, which
/// complete recipients.
fn start_with_accounts(scene: &mut Scene, data: Option<&Path>) {
    let Some(live) = &scene.live else {
        return;
    };
    scene.view.set_accounts(live.summaries());
    if let Some(dir) = data {
        scene.view.set_address_book(address_file::load(dir));
    }
}

/// The lock on the folder this run keeps its accounts in; a second copy is
/// turned away with the name of the first.
fn hold_data_folder(source: &Source) -> Result<Option<DataLock>, AppError> {
    let Some(folder) = source.data_folder() else {
        return Ok(None);
    };
    data_lock::hold(&folder)
}

/// What the Data page is told at start about the vault: the way it is
/// protected (nothing for mock mail, which has no vault) and, on a system this
/// version has no code for a keychain on, that a keychain is not available.
fn tell_how_the_vault_is_kept(view: &mut AppView, source: &Source) {
    if let Some(protection) = protection_of(source) {
        view.set_protection(protection);
    }
    if !engine::vault::keychain_built() {
        view.set_keychain(KeychainState::NotBuilt);
    }
}

/// How the vault of this run is protected, for the Data page; `None` for mock
/// mail, which has no vault.
fn protection_of(source: &Source) -> Option<Protection> {
    let folder = source.secrets_folder()?;
    Some(match engine::vault::SharedVault::for_dir(&folder).protection().mode {
        engine::vault::Mode::Passphrase => Protection::Passphrase,
        engine::vault::Mode::Keychain => Protection::Keychain,
        engine::vault::Mode::KeyFile => Protection::KeyFile,
    })
}

/// What an earlier run that was stopped in the middle of adding or changing an
/// account left in the accounts folder, put right before any account starts.
/// Only a data folder of its own, which this run now holds the lock on, is
/// touched: a folder named by `--account` is shared with other runs.
fn put_right_after_a_hard_stop(source: &Source) -> Vec<String> {
    match source {
        Source::Accounts(root) | Source::Empty(root) => recover(root, &Place::Root(root.clone()).secrets_folder()),
        Source::Mock(_) | Source::Account(_) | Source::Setup(_) => Vec::new(),
    }
}

/// Moves the plain-text secrets of the data folder (the account files'
/// passwords and client secrets, the sign-in tokens and the client secrets of
/// `oauth_clients.toml`) into the encrypted vault, now that this run holds the
/// data folder. Only a data folder of its own is touched, as with
/// `put_right_after_a_hard_stop`.
fn move_secrets(source: &Source) -> Migration {
    match source {
        Source::Accounts(root) => move_secrets_into_vault(root, &Place::Root(root.clone()).secrets_folder()),
        Source::Empty(_) | Source::Mock(_) | Source::Account(_) | Source::Setup(_) => Migration::default(),
    }
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
