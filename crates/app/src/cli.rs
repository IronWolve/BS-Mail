//! The command line: the flags the program takes and what each one means.

use std::path::PathBuf;

use model::config::font::{FONT_SIZE_MAX, FONT_SIZE_MIN, LINE_SPACING_MAX, LINE_SPACING_MIN};
use ui::icons::IconKind;
use ui::layout::{MIN_COLS, MIN_ROWS};

/// The reference grid of the look spec; the size used when none is asked for.
pub const REF_COLS: u16 = 145;
pub const REF_ROWS: u16 = 48;

/// What to do once the arguments are read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Draw,
    Help,
    Version,
    /// Print every setting as it is now, defaults and the user's file
    /// together, as TOML.
    PrintConfig,
}

/// Where the grid size comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeChoice {
    /// The reference grid of the look spec.
    Reference,
    /// The size of the terminal the program was started from.
    Fit,
    Fixed(u16, u16),
}

/// Where the screen is shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// The terminal it was started from; a window only when asked for.
    Auto,
    Terminal,
    Window,
    /// One frame into a picture file, with no window.
    Picture(PathBuf),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    pub action: Action,
    pub mode: Mode,
    pub mock: Option<PathBuf>,
    /// A mail account file: the program runs exactly that account and keeps
    /// its saved sign-in in the file's own folder. Without it the program
    /// runs the accounts in the data folder.
    pub account: Option<PathBuf>,
    /// Show the account form over the accounts that are there. With an
    /// account file the form saves exactly that file and no account runs.
    pub setup: bool,
    /// Where the program keeps its accounts (one folder each, under
    /// `accounts/`) and what it remembers between runs; beside the program
    /// when not given.
    pub data_dir: Option<PathBuf>,
    pub size: SizeChoice,
    /// The icon set, when asked for; otherwise the setting's.
    pub icons: Option<IconKind>,
    pub color: bool,
    pub stats: bool,
    /// Text size in points, when asked for; otherwise the setting's.
    pub points: Option<u8>,
    pub line_spacing: Option<f32>,
    /// Open the window, draw one frame and stop, for a check with nobody
    /// watching.
    pub smoke: bool,
    /// In a terminal, draw once and exit instead of running there.
    pub print: bool,
    /// How many colors the terminal shows, when named; otherwise it is read
    /// from the environment.
    pub color_depth: Option<surface::ColorDepth>,
    /// A `mailto:` link given on the command line: the composer opens for it
    /// once the first account is there.
    pub mailto: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            action: Action::Draw,
            mode: Mode::Auto,
            mock: None,
            account: None,
            setup: false,
            data_dir: None,
            size: SizeChoice::Reference,
            icons: None,
            color: true,
            stats: true,
            points: None,
            line_spacing: None,
            smoke: false,
            print: false,
            color_depth: None,
            mailto: None,
        }
    }
}

/// The grid a window or a picture uses. A terminal's own size is not one of
/// the answers here, so asking to fit gives the reference grid.
pub fn window_grid(size: SizeChoice) -> (u16, u16) {
    let (cols, rows) = match size {
        SizeChoice::Fixed(cols, rows) => (cols, rows),
        SizeChoice::Reference | SizeChoice::Fit => (REF_COLS, REF_ROWS),
    };
    (cols.max(MIN_COLS), rows.max(MIN_ROWS))
}

/// Reads the arguments. `--flag value` and `--flag=value` both work.
/// The folder a flag names. An empty or blank name is refused: taken as a
/// path it means the folder the program happens to run in, and the accounts,
/// the lock and the settings would be written there.
fn folder_value(text: &str, flag: &str) -> Result<PathBuf, String> {
    if text.trim().is_empty() {
        return Err(format!("{flag} needs the name of a folder, not an empty value"));
    }
    Ok(PathBuf::from(text))
}

pub fn parse(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut rest = split_pairs(args).into_iter();
    while let Some(argument) = rest.next() {
        match argument.as_str() {
            "--help" | "-h" => options.action = Action::Help,
            "--version" | "-V" => options.action = Action::Version,
            "--print-config" => options.action = Action::PrintConfig,
            "--window" => options.mode = Mode::Window,
            "--print" => options.print = true,
            "--color-depth" => options.color_depth = Some(parse_depth(&value(&mut rest, &argument)?)?),
            "--smoke" => {
                options.mode = Mode::Window;
                options.smoke = true;
            }
            "--tty" => options.mode = Mode::Terminal,
            "--render-once" => {
                options.mode = Mode::Picture(PathBuf::from(value(&mut rest, &argument)?));
            }
            "--points" => options.points = Some(parse_points(&value(&mut rest, &argument)?)?),
            "--line-spacing" => {
                options.line_spacing = Some(parse_spacing(&value(&mut rest, &argument)?)?);
            }
            "--mock" => options.mock = Some(PathBuf::from(value(&mut rest, &argument)?)),
            "--setup" => options.setup = true,
            "--account" => options.account = Some(PathBuf::from(value(&mut rest, &argument)?)),
            "--data-dir" => options.data_dir = Some(folder_value(&value(&mut rest, &argument)?, &argument)?),
            "--size" => options.size = parse_size(&value(&mut rest, &argument)?)?,
            "--fit" => options.size = SizeChoice::Fit,
            "--icons" => options.icons = Some(parse_icons(&value(&mut rest, &argument)?)?),
            "--no-color" => options.color = false,
            "--quiet" | "-q" => options.stats = false,
            link if is_mailto(link) => options.mailto = Some(link.to_string()),
            other => return Err(format!("unknown option {other}; see --help")),
        }
    }
    // Drawing once is for a terminal or a pipe: it must never end up opening
    // a window because the output happens not to be a console.
    if options.print && options.mode == Mode::Auto {
        options.mode = Mode::Terminal;
    }
    Ok(options)
}

/// Whether an argument is a `mailto:` link, which is the one thing the
/// program takes that is not a flag.
fn is_mailto(argument: &str) -> bool {
    argument.get(..7).is_some_and(|start| start.eq_ignore_ascii_case("mailto:"))
}

/// The value that follows a flag.
fn value(rest: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    rest.next()
        .ok_or_else(|| format!("{flag} needs a value; see --help"))
}

/// Splits every `--flag=value` into two arguments, so one loop handles both
/// spellings.
fn split_pairs(args: &[String]) -> Vec<String> {
    let mut out = Vec::with_capacity(args.len());
    for argument in args {
        match argument.split_once('=') {
            Some((flag, value)) if flag.starts_with("--") => {
                out.push(flag.to_string());
                out.push(value.to_string());
            }
            _ => out.push(argument.clone()),
        }
    }
    out
}

fn parse_size(text: &str) -> Result<SizeChoice, String> {
    let (cols, rows) = text
        .split_once(['x', 'X'])
        .ok_or_else(|| format!("--size wants COLSxROWS, not {text}"))?;
    let number = |part: &str, what: &str| {
        part.trim()
            .parse::<u16>()
            .map_err(|_| format!("--size {what} is not a number: {part}"))
    };
    Ok(SizeChoice::Fixed(
        number(cols, "width")?,
        number(rows, "height")?,
    ))
}

fn parse_depth(text: &str) -> Result<surface::ColorDepth, String> {
    tty_host::parse_depth(text)
        .ok_or_else(|| format!("--color-depth wants truecolor, 256, 16 or none, not {text}"))
}

fn parse_points(text: &str) -> Result<u8, String> {
    let points: u8 = text
        .trim()
        .parse()
        .map_err(|_| format!("--points is not a number: {text}"))?;
    if !(FONT_SIZE_MIN..=FONT_SIZE_MAX).contains(&points) {
        return Err(format!(
            "--points wants {FONT_SIZE_MIN} to {FONT_SIZE_MAX}, not {points}"
        ));
    }
    Ok(points)
}

fn parse_spacing(text: &str) -> Result<f32, String> {
    let spacing: f32 = text
        .trim()
        .parse()
        .map_err(|_| format!("--line-spacing is not a number: {text}"))?;
    if !(LINE_SPACING_MIN..=LINE_SPACING_MAX).contains(&spacing) {
        return Err(format!(
            "--line-spacing wants {LINE_SPACING_MIN} to {LINE_SPACING_MAX}, not {spacing}"
        ));
    }
    Ok(spacing)
}

fn parse_icons(text: &str) -> Result<IconKind, String> {
    match text {
        "nerd" => Ok(IconKind::Nerd),
        "unicode" => Ok(IconKind::Unicode),
        "ascii" => Ok(IconKind::Ascii),
        other => Err(format!("--icons wants nerd, unicode or ascii, not {other}")),
    }
}

#[cfg(test)]
include!("cli.tests.rs");
