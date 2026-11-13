//! What the program says about itself: the help text, the version line and
//! the stats block after a run. All of it goes to standard error, so the
//! screen on standard output can be piped or saved on its own.

use std::io::{IsTerminal, Write};

use crate::cli::{REF_COLS, REF_ROWS};
use crate::identity::{display_name, slug, version};
use crate::paths::FIXTURE_NAME;
use crate::run::Stats;

struct Paint {
    bold: &'static str,
    green: &'static str,
    red: &'static str,
    dim: &'static str,
    reset: &'static str,
}

fn paint() -> Paint {
    let plain = std::env::var_os("NO_COLOR").is_some() || !std::io::stderr().is_terminal();
    if plain {
        Paint {
            bold: "",
            green: "",
            red: "",
            dim: "",
            reset: "",
        }
    } else {
        Paint {
            bold: "\x1b[1m",
            green: "\x1b[32m",
            red: "\x1b[31m",
            dim: "\x1b[2m",
            reset: "\x1b[0m",
        }
    }
}

/// One failure, on standard error.
pub fn print_error(message: &str) {
    let paint = paint();
    let _ = writeln!(
        std::io::stderr().lock(),
        "{}{}: {message}{}",
        paint.red,
        slug(),
        paint.reset
    );
}

pub fn version_line() -> String {
    format!("{} {}", display_name(), version())
}

pub fn help_text() -> String {
    let name = slug();
    format!(
        "{} {} - draws one screen of mock mail and exits.\n\
         \n\
         Usage: {name} [options]\n\
         \n\
         Options:\n  \
         --mock <file>      mock data to draw (default: {FIXTURE_NAME} beside the program)\n  \
         --size <COLSxROWS> grid to draw (default: {REF_COLS}x{REF_ROWS}, the reference size)\n  \
         --fit              use the size of this terminal instead\n  \
         --icons <set>      nerd, unicode or ascii (default: nerd)\n  \
         --no-color         write the symbols without any color\n  \
         --quiet, -q        leave out the stats block\n  \
         --version, -V      print the version\n  \
         --help, -h         print this text\n\
         \n\
         The window is not built yet: this draws the same cells into the\n\
         terminal, without the double-size banner and the rounded shapes.\n",
        display_name(),
        version(),
    )
}

pub fn print_stats(stats: &Stats) {
    let paint = paint();
    let mut out = std::io::stderr().lock();
    let _ = writeln!(out, "\n{}Screen stats{}", paint.bold, paint.reset);
    let mut line = |label: &str, color: &str, value: String| {
        let _ = writeln!(out, "  {label:<12} {color}{value}{}", paint.reset);
    };
    line(
        "grid",
        paint.green,
        format!("{} x {} cells", stats.cols, stats.rows),
    );
    line("messages", paint.green, stats.messages.to_string());
    line("effects", paint.green, format!("{} (not drawn here)", stats.effects));
    line("drawn in", paint.green, format!("{} ms", stats.millis));
    line("mock data", paint.dim, stats.fixture.display().to_string());
}
