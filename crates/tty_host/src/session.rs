//! Taking over the terminal, and giving it back.
//!
//! Starting switches to the alternate screen, turns on raw input, mouse
//! reports, bracketed paste and focus reports, and hides the cursor. Giving
//! it back undoes all of that. It happens when the session is dropped, when
//! the program panics, and when the terminal is hung up or told to terminate,
//! so the user's shell is never left in raw mode on a blank screen.

use std::io::{self, Write};
use std::sync::Arc;
use std::sync::Once;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crossterm::cursor::{Hide, Show};
use crossterm::event::{
    DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
    EnableFocusChange, EnableMouseCapture,
};
use crossterm::execute;
use crossterm::terminal::{
    DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode,
};

/// Whether the terminal is ours right now, so it is given back only once.
static ACTIVE: AtomicBool = AtomicBool::new(false);
static HOOK: Once = Once::new();

pub struct Session {
    /// Set when the terminal is hung up or told to terminate.
    pub stop: Arc<AtomicBool>,
    /// Set when the loop has left, so the watch knows it is not stuck.
    left: Arc<AtomicBool>,
}

impl Session {
    pub fn start() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut out = io::stdout();
        let started = execute!(
            out,
            EnterAlternateScreen,
            DisableLineWrap,
            EnableMouseCapture,
            EnableBracketedPaste,
            EnableFocusChange,
            Hide
        );
        if let Err(error) = started {
            ACTIVE.store(true, Ordering::SeqCst);
            restore();
            return Err(error);
        }
        ACTIVE.store(true, Ordering::SeqCst);
        install_panic_hook();
        let stop = stop_flag();
        let left = Arc::new(AtomicBool::new(false));
        watch_for_stop(Arc::clone(&stop), Arc::clone(&left));
        Ok(Self { stop, left })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        restore();
        self.left.store(true, Ordering::Release);
    }
}

/// Gives the terminal back. Safe to call any number of times.
pub fn restore() {
    if !ACTIVE.swap(false, Ordering::SeqCst) {
        return;
    }
    let mut out = io::stdout();
    let _ = execute!(
        out,
        DisableFocusChange,
        DisableBracketedPaste,
        DisableMouseCapture,
        Show,
        EnableLineWrap,
        LeaveAlternateScreen
    );
    // The cursor shape and the colors back to the terminal's own.
    let _ = out.write_all(b"\x1b[0 q\x1b[0m");
    let _ = out.flush();
    let _ = disable_raw_mode();
}

/// A panic gives the terminal back before it prints, so the message can be
/// read instead of being lost on the alternate screen.
fn install_panic_hook() {
    HOOK.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore();
            previous(info);
        }));
    });
}

/// How long the loop is given to leave by itself after a stop was asked for.
const LEAVE_GRACE: Duration = Duration::from_millis(1000);

/// How often the watch looks at the stop flag.
const WATCH_EVERY: Duration = Duration::from_millis(100);

/// How long the program is given to finish once the loop has left: it saves
/// its state and its backend waits for a send that is still on its way (up to
/// eight seconds) before the program ends by itself.
const FINISH_GRACE: Duration = Duration::from_millis(12_000);

/// What the watch found when a stop was asked for.
#[derive(Debug, PartialEq, Eq)]
enum Ending {
    /// The loop did not leave in time: the terminal's wait never returns.
    Stuck,
    /// The loop left but the program had still not ended after the time it is
    /// given to finish.
    TooSlow,
}

/// Waits for a stop to be asked for and then says how it went. The loop looks
/// at the flag between waits for input, but once a terminal is hung up the
/// terminal library's wait can go on forever at full speed, so the loop may
/// never get to look: after `leave` it is `Stuck`. A loop that did leave is
/// not stuck: the program is saving and its backend is letting a send finish,
/// so it is given `finish` more before it is `TooSlow`.
fn wait_for_ending(stop: &AtomicBool, left: &AtomicBool, every: Duration, leave: Duration, finish: Duration) -> Ending {
    while !stop.load(Ordering::Acquire) {
        std::thread::sleep(every);
    }
    std::thread::sleep(leave);
    if !left.load(Ordering::Acquire) {
        return Ending::Stuck;
    }
    let until = std::time::Instant::now() + finish;
    while std::time::Instant::now() < until {
        std::thread::sleep(every);
    }
    Ending::TooSlow
}

/// Ends the program when a stop was asked for and it does not end by itself
/// (see `wait_for_ending`). A program that ends in time takes this thread
/// with it.
fn watch_for_stop(stop: Arc<AtomicBool>, left: Arc<AtomicBool>) {
    let _ = std::thread::Builder::new().name("terminal-stop".to_owned()).spawn(move || {
        wait_for_ending(&stop, &left, WATCH_EVERY, LEAVE_GRACE, FINISH_GRACE);
        restore();
        std::process::exit(STOPPED_EXIT_CODE);
    });
}

/// The exit code when the program was ended from the watch: 128 + SIGHUP.
const STOPPED_EXIT_CODE: i32 = 129;

/// A flag that is set when the terminal is hung up or told to terminate. The
/// loop looks at it and leaves in the ordinary way.
fn stop_flag() -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    for signal in [signal_hook::consts::SIGTERM, signal_hook::consts::SIGHUP] {
        let _ = signal_hook::flag::register(signal, flag.clone());
    }
    flag
}

#[cfg(test)]
include!("session.tests.rs");
