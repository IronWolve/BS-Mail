//! What the program says about itself: the help text, the version line and
//! the stats block after a run. All of it goes to standard error, so the
//! screen on standard output can be piped or saved on its own.

use std::io::{IsTerminal, Write};

use crate::cli::{REF_COLS, REF_ROWS};
use crate::identity::{display_name, slug, version};
use crate::paths::{ACCOUNT_FILE, ACCOUNTS_DIR_NAME};
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
        "{} {} - a mail client.\n\
         \n\
         Usage: {name} [options] [mailto:address]\n\
         \n\
         Where it draws (default: a window when started on its own, this\n\
         terminal when started from one):\n  \
         --window           open a window and draw there\n  \
         --tty              run in this terminal (with --print: draw once and exit)\n  \
         --print            draw once into the terminal or a pipe and exit (never a window)\n  \
         --color-depth <d>  truecolor, 256, 16 or none (default: from the environment)\n  \
         --render-once <f>  draw once into a picture file and exit\n  \
         --smoke            open a window, draw one frame and stop\n\
         \n\
         Options:\n  \
         --account <file>   run only the account in this file; its saved sign-in is kept in the\n                     file's own folder (default: every account in\n                     <data dir>/{ACCOUNTS_DIR_NAME}/<id>/{ACCOUNT_FILE})\n  \
         --setup            show the account form over the accounts that are there (with\n                     --account <file>: the form saves exactly that file and no account runs\n                     until it has)\n  \
         --data-dir <dir>   where accounts, settings and sizes are kept (default: data beside\n                     the program); each account has a folder of its own in it\n  \
         --mock <file>      draw the mock mail in this file instead of an account (for trying the look)\n  \
         --size <COLSxROWS> grid to draw (default: {REF_COLS}x{REF_ROWS}, the reference size)\n  \
         --fit              use the size of this terminal instead\n  \
         --points <n>       text size of the window and of pictures (default: font.size)\n  \
         --line-spacing <n> row height, 0.9 to 1.6 (default: font.line_spacing)\n  \
         --icons <set>      nerd, unicode or ascii (default: nerd)\n  \
         --no-color         write the symbols without any color\n  \
         --quiet, -q        leave out the stats block\n  \
         --print-config     print every setting as it is now, as TOML\n  \
         mailto:<address>   start a new message to this address (and with the subject and text\n                     of the link, when it has them) once the first account is there\n  \
         --version, -V      print the version\n  \
         --help, -h         print this text\n\
         \n\
         In the window: arrows, Page Up and Down, Home and End move in the pane\n\
         that has focus; Tab changes pane, Enter goes in, Escape comes back; a\n\
         click selects and the wheel scrolls. u marks read or unread, * stars,\n\
         d deletes, s syncs; a message opened for a moment is marked read.\n\
         These change mail only for an account with read_only = false.\n\
         Ctrl and plus or minus, or Ctrl\n\
         and the wheel, change the text size, Ctrl 0 puts it back, F11 is full\n\
         screen, q or Ctrl Q closes it.\n\
         The terminal run takes the same keys and the mouse, and leaves out the\n\
         double-size banner title and the rounded shapes, which only the\n\
         window can draw. Ctrl+C is a key in the terminal run: quit with q.\n",
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
    match stats.accounts {
        Some(count) => line("accounts", paint.green, count.to_string()),
        None => line("messages", paint.green, stats.messages.to_string()),
    }
    match &stats.shown {
        Shown::Tty { frames, depth } => {
            line("frames", paint.green, frames.to_string());
            line("colors", paint.green, (*depth).to_string());
        }
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
    line(stats.label, paint.dim, stats.source.display().to_string());
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

#[cfg(test)]
include!("report.tests.rs");
