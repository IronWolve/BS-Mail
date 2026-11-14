//! The terminal as a place to show frames and take input.
//!
//! `TtyHost` paints the screen's frames into the terminal it was started in,
//! changed cells only, and runs the loop that turns terminal input into the
//! screen's events. It is the terminal's counterpart of the window host: the
//! screen code cannot tell them apart except through what each can do.

use std::io::{self, BufWriter, Stdout, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use surface::{Capabilities, ColorDepth, EventSink, Frame, HostError, Presenter, Waker};

use crate::draw::Painter;
use crate::event_loop;
use crate::osc::{clipboard_sequence, title_sequence};

pub struct TtyHost {
    painter: Painter,
    depth: ColorDepth,
    out: BufWriter<Stdout>,
    redraw: bool,
    /// Set by another thread that wants the loop to look for news.
    wake: Arc<AtomicBool>,
    pub(crate) failure: Option<HostError>,
}

impl TtyHost {
    pub fn new(depth: ColorDepth) -> Self {
        Self {
            painter: Painter::new(depth),
            depth,
            out: BufWriter::new(io::stdout()),
            redraw: false,
            wake: Arc::new(AtomicBool::new(false)),
            failure: None,
        }
    }

    /// Asks for a draw without going through the screen, for the first frame.
    pub(crate) fn request_redraw_now(&mut self) {
        self.redraw = true;
    }

    pub(crate) fn take_redraw(&mut self) -> bool {
        std::mem::take(&mut self.redraw)
    }

    pub(crate) fn take_wake(&mut self) -> bool {
        self.wake.swap(false, Ordering::AcqRel)
    }

    /// The terminal's contents may no longer be what was drawn, so the next
    /// frame repaints everything.
    pub(crate) fn forget(&mut self) {
        self.painter.forget();
    }

    fn write(&mut self, bytes: &[u8]) {
        let result = self.out.write_all(bytes).and_then(|()| self.out.flush());
        if let Err(error) = result {
            self.failure = Some(HostError(format!("writing to the terminal failed: {error}")));
        }
    }
}

impl Presenter for TtyHost {
    fn capabilities(&self) -> Capabilities {
        Capabilities::terminal(self.depth)
    }

    fn present(&mut self, frame: &Frame) {
        if let Err(error) = self.painter.paint(frame, &mut self.out) {
            self.failure = Some(HostError(format!("writing to the terminal failed: {error}")));
        }
    }

    /// A terminal has one pointer, which the terminal itself owns.
    fn set_pointer_shape(&mut self, _shape: surface::PointerShape) {}

    fn set_clipboard(&mut self, text: &str) {
        self.write(clipboard_sequence(text).as_bytes());
    }

    fn set_title(&mut self, title: &str) {
        self.write(title_sequence(title).as_bytes());
    }

    fn request_redraw(&mut self) {
        self.redraw = true;
    }

    fn waker(&self) -> Waker {
        let flag = self.wake.clone();
        Waker::new(move || flag.store(true, Ordering::Release))
    }

    fn run(&mut self, sink: &mut dyn EventSink) -> Result<(), HostError> {
        event_loop::run(self, sink)
    }
}
