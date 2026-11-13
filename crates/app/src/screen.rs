//! The screen on the other end of a host's loop: it answers events and hands
//! back a frame whenever the host asks for one.
//!
//! There is no key map yet, so only the keys that close a window are read
//! here; everything else is accepted and ignored.

use surface::{EventSink, Flow, InputEvent, KeyEvent, LogicalKey, NamedKey, Presenter};
use ui::AppView;

pub struct Screen {
    view: AppView,
    cols: u16,
    rows: u16,
    frames: u32,
    /// Frames after which the run stops itself, for a check with nobody
    /// watching; `None` runs until the window is closed.
    stop_after: Option<u32>,
    done: bool,
}

impl Screen {
    pub fn new(view: AppView, cols: u16, rows: u16) -> Self {
        Self {
            view,
            cols,
            rows,
            frames: 0,
            stop_after: None,
            done: false,
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
}

impl EventSink for Screen {
    fn on_input(&mut self, event: InputEvent, host: &mut dyn Presenter) -> Flow {
        match event {
            InputEvent::Resized { cols, rows } => {
                self.cols = cols;
                self.rows = rows;
                host.request_redraw();
                Flow::Continue
            }
            InputEvent::CloseRequested => Flow::Quit,
            InputEvent::Key(key) if closes(&key) => Flow::Quit,
            _ => Flow::Continue,
        }
    }

    fn on_wake(&mut self, _host: &mut dyn Presenter) -> Flow {
        if self.done { Flow::Quit } else { Flow::Continue }
    }

    fn on_redraw(&mut self, host: &mut dyn Presenter) {
        self.view.set_capabilities(host.capabilities());
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

/// The keys that close the window while there is no key map.
fn closes(key: &KeyEvent) -> bool {
    match key.key {
        LogicalKey::Named(NamedKey::Escape) => true,
        LogicalKey::Char('q') => key.modifiers.ctrl,
        _ => false,
    }
}

#[cfg(test)]
include!("screen.tests.rs");
