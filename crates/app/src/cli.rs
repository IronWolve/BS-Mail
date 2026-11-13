//! The command line: the flags the program takes and what each one means.

use std::path::PathBuf;

use ui::icons::IconKind;

/// The reference grid of the look spec; the size used when none is asked for.
pub const REF_COLS: u16 = 145;
pub const REF_ROWS: u16 = 48;

/// What to do once the arguments are read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Draw,
    Help,
    Version,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub action: Action,
    pub mock: Option<PathBuf>,
    pub size: SizeChoice,
    pub icons: IconKind,
    pub color: bool,
    pub stats: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            action: Action::Draw,
            mock: None,
            size: SizeChoice::Reference,
            icons: IconKind::Nerd,
            color: true,
            stats: true,
        }
    }
}

/// Reads the arguments. `--flag value` and `--flag=value` both work.
pub fn parse(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut rest = split_pairs(args).into_iter();
    while let Some(argument) = rest.next() {
        match argument.as_str() {
            "--help" | "-h" => options.action = Action::Help,
            "--version" | "-V" => options.action = Action::Version,
            "--mock" => options.mock = Some(PathBuf::from(value(&mut rest, &argument)?)),
            "--size" => options.size = parse_size(&value(&mut rest, &argument)?)?,
            "--fit" => options.size = SizeChoice::Fit,
            "--icons" => options.icons = parse_icons(&value(&mut rest, &argument)?)?,
            "--no-color" => options.color = false,
            "--quiet" | "-q" => options.stats = false,
            other => return Err(format!("unknown option {other}; see --help")),
        }
    }
    Ok(options)
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
