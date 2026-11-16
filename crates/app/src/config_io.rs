//! Reading the user's settings: `config.toml` and a named theme, from the
//! data folder.
//!
//! A missing file is not a problem: the defaults are the program. A file
//! that cannot be read, a value out of range and a theme that cannot be found
//! each give the default for that one thing and a problem to show the user,
//! so a typo costs one setting and never the program.

use std::fs;
use std::path::Path;

use model::config::Config;
use model::config::font::{FONT_SIZE_MAX, FONT_SIZE_MIN, LINE_SPACING_MAX, LINE_SPACING_MIN};
use ui::keys::KeyMap;
use ui::theme::{DEFAULT_THEME_NAME, Theme, builtin_theme_text, default_theme, load_theme_file};

use crate::errors::AppError;
use crate::unused_settings;

pub const CONFIG_NAME: &str = "config.toml";
pub const THEMES_DIR: &str = "themes";

/// What was read, and what was wrong with it.
pub struct Loaded {
    pub config: Config,
    pub theme: Theme,
    /// One line each, for the footer.
    pub problems: Vec<String>,
    /// Set when `config.toml` is there but could not be read or understood:
    /// what was wrong, without what became of the settings. The config is
    /// then the defaults, which a caller that has settings in force already
    /// should not take.
    pub file_problem: Option<String>,
}

/// Reads the settings in `dir`, or the defaults when there is no folder.
pub fn load(dir: Option<&Path>) -> Result<Loaded, AppError> {
    let mut problems = Vec::new();
    let read = dir.map_or_else(|| Ok(Config::default()), read_config);
    let (mut config, file_problem) = match read {
        Ok(config) => (config, None),
        Err(reason) => {
            problems.push(format!("{reason}; using the defaults"));
            (Config::default(), Some(reason))
        }
    };
    clamp(&mut config, &mut problems);
    let theme = pick_theme(&config.appearance.theme, dir, &mut problems)?;
    // Last, so a problem that changed a value is the one the footer shows.
    problems.extend(unused_settings::note(&config));
    problems.extend(KeyMap::new(&config.keys).problems().iter().cloned());
    Ok(Loaded { config, theme, problems, file_problem })
}

/// The one line the footer shows for `problems`: the first, and how many more
/// there are, so a second mistake is not hidden behind the first.
pub fn footer_note(problems: &[String]) -> Option<String> {
    let first = problems.first()?;
    match problems.len() {
        1 => Some(first.clone()),
        more => Some(format!("{first} (and {} more: --print-config lists them)", more - 1)),
    }
}

/// The file's settings, or what was wrong with it. No file is no problem.
fn read_config(dir: &Path) -> Result<Config, String> {
    let path = dir.join(CONFIG_NAME);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(error) => return Err(format!("{CONFIG_NAME} could not be read: {error}")),
    };
    toml::from_str::<Config>(&text).map_err(|error| {
        // The message says what; the span says where. The rest of the
        // error quotes the file, which does not belong in a footer.
        let where_ = error.span().map_or(String::new(), |span| format!(" (line {})", line_of(&text, span.start)));
        format!("{CONFIG_NAME}{where_}: {}", error.message().trim())
    })
}

fn line_of(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].matches('\n').count() + 1
}

/// Brings every ranged value inside its range, saying so for each.
fn clamp(config: &mut Config, problems: &mut Vec<String>) {
    let size = config.font.size.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX);
    if size != config.font.size {
        problems.push(format!("font.size {} is out of range, using {size}", config.font.size));
        config.font.size = size;
    }
    let given = config.font.line_spacing;
    // The file can say `nan`; it is no number, so no comparison with it holds
    // and it has to be caught before the one that checks for a change.
    let spacing = if given.is_nan() { 1.0 } else { given.clamp(LINE_SPACING_MIN, LINE_SPACING_MAX) };
    if given.is_nan() || (spacing - given).abs() > f32::EPSILON {
        problems.push(format!("font.line_spacing {given} is out of range, using {spacing}"));
        config.font.line_spacing = spacing;
    }
    if ui::list_order::Sort::from_setting(&config.list.default_sort.column, config.list.default_sort.descending).is_none() {
        let name = config.list.default_sort.column.clone();
        problems.push(format!("list.default_sort: `{name}` is not a column to sort by (date, from, subject, size or flag); using date"));
        config.list.default_sort = model::config::list::SortSetting::default();
    }
    let weight = config.appearance.line_weight.clamp(1, 4);
    if weight != config.appearance.line_weight {
        problems.push(format!("appearance.line_weight {} is out of range, using {weight}", config.appearance.line_weight));
        config.appearance.line_weight = weight;
    }
}

/// The theme at start: the one the settings name, or the built-in one with a
/// problem to show when that cannot be had.
fn pick_theme(name: &str, dir: Option<&Path>, problems: &mut Vec<String>) -> Result<Theme, AppError> {
    let default = || default_theme().map_err(|error| AppError::Theme(error.to_string()));
    if name == DEFAULT_THEME_NAME || name.is_empty() {
        return default();
    }
    match theme_named(name, dir) {
        Ok(theme) => Ok(theme),
        Err(reason) => {
            problems.push(format!("{reason}; using {DEFAULT_THEME_NAME}"));
            default()
        }
    }
}

/// The theme called `name`: the built-in one, or `themes/<name>.toml` in the
/// data folder; another theme it inherits from is found the same way. A theme
/// that is not there or does not load gives one line saying why, starting
/// with `theme <name>`.
///
/// The theme goes by the name it was asked for, which is what the settings
/// file keeps, whatever name its own file gives it or inherits.
pub fn theme_named(name: &str, dir: Option<&Path>) -> Result<Theme, String> {
    let Some(text) = find_theme_text(name, dir) else {
        return Err(format!("theme {name} was not found in {THEMES_DIR}/"));
    };
    let found = |wanted: &str| find_theme_text(wanted, dir);
    let failed = |error: &dyn std::fmt::Display| {
        let first = error.to_string().lines().next().unwrap_or_default().to_string();
        format!("theme {name}: {first}")
    };
    let mut file = load_theme_file(&text, &found).map_err(|error| failed(&error))?;
    file.name = name.to_string();
    Theme::from_file(&file).map_err(|error| failed(&error))
}

/// A theme's text by name. A name that could point outside the themes folder
/// is not a theme.
fn find_theme_text(name: &str, dir: Option<&Path>) -> Option<String> {
    if let Some(text) = builtin_theme_text(name) {
        return Some(text.to_string());
    }
    if !is_plain_name(name) {
        return None;
    }
    fs::read_to_string(dir?.join(THEMES_DIR).join(format!("{name}.toml"))).ok()
}

/// Whether a theme name is only letters, digits, `-` and `_`.
pub(crate) fn is_plain_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
}

#[cfg(test)]
include!("config_io.tests.rs");
