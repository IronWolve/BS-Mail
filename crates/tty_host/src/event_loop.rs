//! The terminal's loop: wait for input, hand it to the screen, draw when the
//! screen asks, and wake the screen now and then so it can look for news from
//! the mail backend.

use std::sync::atomic::Ordering;
use std::time::Duration;

use crossterm::event;
use crossterm::terminal;
use surface::{EventSink, Flow, HostError, InputEvent};

use crate::host::TtyHost;
use crate::input::translate;
use crate::session::Session;

/// How long to wait for input before the screen is woken anyway.
const TICK: Duration = Duration::from_millis(100);

pub fn run(host: &mut TtyHost, sink: &mut dyn EventSink) -> Result<(), HostError> {
    let session = Session::start().map_err(|error| HostError(format!("the terminal could not be taken over: {error}")))?;
    let (cols, rows) = terminal::size().map_err(|error| HostError(format!("the terminal's size is unknown: {error}")))?;
    if sink.on_input(InputEvent::Resized { cols, rows }, host) == Flow::Quit {
        return Ok(());
    }
    host.request_redraw_now();
    loop {
        if host.take_resumed() {
            // The terminal may have been resized while the program was stopped.
            let (cols, rows) = terminal::size().map_err(|error| HostError(format!("the terminal's size is unknown: {error}")))?;
            if sink.on_input(InputEvent::Resized { cols, rows }, host) == Flow::Quit {
                return Ok(());
            }
        }
        if host.take_redraw() {
            sink.on_redraw(host);
        }
        if let Some(failure) = host.failure.take() {
            return Err(failure);
        }
        if session.stop.load(Ordering::Acquire) {
            return Ok(());
        }
        let waited = event::poll(TICK).map_err(read_error)?;
        if waited && handle_pending(host, sink)? == Flow::Quit {
            return Ok(());
        }
        // Something outside asked for a look, or the wait ran out: the screen
        // checks the backend either way.
        if (!waited || host.take_wake()) && sink.on_wake(host) == Flow::Quit {
            return Ok(());
        }
    }
}

/// Reads every event that is ready and delivers it.
fn handle_pending(host: &mut TtyHost, sink: &mut dyn EventSink) -> Result<Flow, HostError> {
    loop {
        let event = event::read().map_err(read_error)?;
        if let event::Event::Resize(..) = event {
            // The terminal redraws nothing itself after a resize.
            host.forget();
        }
        for translated in translate(&event) {
            if sink.on_input(translated, host) == Flow::Quit {
                return Ok(Flow::Quit);
            }
        }
        if !event::poll(Duration::ZERO).map_err(read_error)? {
            return Ok(Flow::Continue);
        }
    }
}

fn read_error(error: std::io::Error) -> HostError {
    HostError(format!("reading from the terminal failed: {error}"))
}
