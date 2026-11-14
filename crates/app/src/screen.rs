//! The screen on the other end of a host's loop: it answers events and hands
//! back a frame whenever the host asks for one.
//!
//! Input goes to the view, which moves the selection; the commands that
//! selection needs go to the backend; what the backend says comes back to the
//! view. Only the keys that close the program are read here.

use std::time::Instant;

use model::{MailBackend, MailDate};
use surface::{EventSink, Flow, InputEvent, KeyEvent, LogicalKey, Presenter};
use ui::AppView;

use crate::clock::system_now;
use crate::pump::pump;
use crate::ticker::Ticker;

pub struct Screen {
    view: AppView,
    backend: Box<dyn MailBackend>,
    cols: u16,
    rows: u16,
    frames: u32,
    /// A clock that stands still; `None` follows the system clock.
    fixed_now: Option<MailDate>,
    /// Frames after which the run stops itself, for a check with nobody
    /// watching; `None` runs until the window is closed.
    stop_after: Option<u32>,
    done: bool,
    ticker: Option<Ticker>,
    /// When the screen started, for the steady clock the view's timers use.
    started: Instant,
}

impl Screen {
    pub fn new(
        view: AppView,
        backend: Box<dyn MailBackend>,
        grid: (u16, u16),
        fixed_now: Option<MailDate>,
    ) -> Self {
        Self {
            view,
            backend,
            cols: grid.0,
            rows: grid.1,
            frames: 0,
            fixed_now,
            stop_after: None,
            done: false,
            ticker: None,
            started: Instant::now(),
        }
    }

    /// Stops the run once `frames` frames have been drawn.
    pub fn stop_after(mut self, frames: u32) -> Self {
        self.stop_after = Some(frames.max(1));
        self
    }

    /// How many frames were drawn, for the stats block.
    pub fn frames(&self) -> u32 {
        self.frames
    }

    pub fn grid(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    /// Moves mail both ways until quiet; returns whether the view changed.
    fn exchange(&mut self) -> bool {
        pump(&mut self.view, self.backend.as_mut()) > 0
    }
}

impl EventSink for Screen {
    fn on_input(&mut self, event: InputEvent, host: &mut dyn Presenter) -> Flow {
        match &event {
            InputEvent::Resized { cols, rows } => {
                self.cols = *cols;
                self.rows = *rows;
                host.request_redraw();
                return Flow::Continue;
            }
            InputEvent::CloseRequested => return Flow::Quit,
            InputEvent::Key(key) if closes(key) => return Flow::Quit,
            InputEvent::PointerMoved { pos, .. } => {
                let shape = self.view.pointer_shape(pos.col, pos.row, (self.cols, self.rows));
                host.set_pointer_shape(shape);
            }
            _ => {}
        }
        if self.view.handle_input(&event, (self.cols, self.rows)) {
            self.exchange();
            host.request_redraw();
        }
        Flow::Continue
    }

    fn on_wake(&mut self, host: &mut dyn Presenter) -> Flow {
        if self.done {
            return Flow::Quit;
        }
        let elapsed = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.view.tick(elapsed);
        if self.exchange() {
            host.request_redraw();
        }
        Flow::Continue
    }

    fn on_redraw(&mut self, host: &mut dyn Presenter) {
        if self.ticker.is_none() {
            self.ticker = Some(Ticker::start(host.waker()));
        }
        self.view.set_capabilities(host.capabilities());
        self.view.set_now(self.fixed_now.unwrap_or_else(system_now));
        let frame = self.view.render(self.cols, self.rows);
        host.present(&frame);
        self.frames = self.frames.saturating_add(1);
        if self.stop_after.is_some_and(|limit| self.frames >= limit) {
            // Asking the host's own loop to come back lets it finish showing
            // this frame before the run ends.
            self.done = true;
            host.waker().wake();
        }
    }
}

/// The keys that close the program: `q`, as the key bar says, and `Ctrl+Q`.
fn closes(key: &KeyEvent) -> bool {
    let held = key.modifiers;
    match key.key {
        LogicalKey::Char('q') => !held.alt && !held.meta && (held.ctrl || !held.shift),
        _ => false,
    }
}

#[cfg(test)]
include!("screen.tests.rs");
