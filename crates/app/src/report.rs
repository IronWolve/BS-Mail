//! What the program says about itself: the help text, the version line and
//! the stats block after a run. All of it goes to standard error, so the
//! screen on standard output can be piped or saved on its own.

use std::io::{IsTerminal, Write};

use crate::cli::{DEFAULT_POINTS, REF_COLS, REF_ROWS};
use crate::identity::{display_name, slug, version};
use crate::paths::FIXTURE_NAME;
use crate::stats::{Shown, Stats};

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
        "{} {} - draws one screen of mock mail.\n\
         \n\
         Usage: {name} [options]\n\
         \n\
         Where it draws (default: a window when started on its own, this\n\
         terminal when started from one):\n  \
         --window           open a window and draw there\n  \
         --tty              draw in this terminal and exit\n  \
         --render-once <f>  draw once into a picture file and exit\n  \
         --smoke            open a window, draw one frame and stop\n\
         \n\
         Options:\n  \
         --mock <file>      mock data to draw (default: {FIXTURE_NAME} beside the program)\n  \
         --size <COLSxROWS> grid to draw (default: {REF_COLS}x{REF_ROWS}, the reference size)\n  \
         --fit              use the size of this terminal instead\n  \
         --points <n>       text size of the window and of pictures (default: {DEFAULT_POINTS})\n  \
         --line-spacing <n> row height, 0.9 to 1.6 (default: 1.0)\n  \
         --icons <set>      nerd, unicode or ascii (default: nerd)\n  \
         --no-color         write the symbols without any color\n  \
         --quiet, -q        leave out the stats block\n  \
         --version, -V      print the version\n  \
         --help, -h         print this text\n\
         \n\
         In the window: arrows, Page Up and Down, Home and End move in the pane\n\
         that has focus; Tab changes pane, Enter goes in, Escape comes back; a\n\
         click selects and the wheel scrolls. Ctrl and plus or minus, or Ctrl\n\
         and the wheel, change the text size, Ctrl 0 puts it back, F11 is full\n\
         screen, q or Ctrl Q closes it.\n\
         The terminal draw leaves out the double-size banner title and the\n\
         rounded shapes, which only the window can draw.\n",
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
    match &stats.shown {
        Shown::Terminal => {
            line("effects", paint.green, format!("{} (not drawn here)", stats.effects));
        }
        Shown::Window {
            frames,
            points,
            font_bytes,
        } => {
            line("effects", paint.green, stats.effects.to_string());
            line("frames", paint.green, frames.to_string());
            line("text size", paint.green, format!("{points} points"));
            line("fonts", paint.dim, format!("{} built in", bytes(*font_bytes as u64)));
        }
        Shown::Picture {
            path,
            width,
            height,
            bytes: size,
        } => {
            line("effects", paint.green, stats.effects.to_string());
            line("picture", paint.green, format!("{width} x {height} pixels"));
            line("file", paint.green, format!("{} ({})", path.display(), bytes(*size)));
        }
    }
    line("drawn in", paint.green, format!("{} ms", stats.millis));
    line("mock data", paint.dim, stats.fixture.display().to_string());
}

/// A byte count in the largest unit that keeps it short.
fn bytes(count: u64) -> String {
    const STEP: f64 = 1024.0;
    let value = count as f64;
    if value < STEP {
        return format!("{count} B");
    }
    if value < STEP * STEP {
        return format!("{:.1} KB", value / STEP);
    }
    format!("{:.1} MB", value / (STEP * STEP))
}
