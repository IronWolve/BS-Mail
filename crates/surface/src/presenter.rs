//! The one trait both hosts implement: show frames, report capabilities,
//! deliver input events, take pointer-shape and redraw requests.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::capabilities::Capabilities;
use crate::frame::Frame;
use crate::input_event::InputEvent;
use crate::pointer_shape::PointerShape;

/// A failure inside a host that ends its run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostError(pub String);

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for HostError {}

/// Whether the host's loop keeps going after a callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Continue,
    Quit,
}

/// A cloneable handle other threads use to wake the host's loop; the host then
/// calls `EventSink::on_wake` on its own thread.
#[derive(Clone)]
pub struct Waker(Arc<dyn Fn() + Send + Sync>);

impl Waker {
    pub fn new(wake: impl Fn() + Send + Sync + 'static) -> Self {
        Self(Arc::new(wake))
    }

    pub fn wake(&self) {
        (self.0)();
    }
}

impl fmt::Debug for Waker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Waker")
    }
}

/// The UI side of the loop. The host calls these on its own thread and passes
/// itself back so the callback can present a frame or change the pointer.
pub trait EventSink {
    /// One normalized input event. A host delivers `Resized` with the initial
    /// grid size before its first redraw.
    fn on_input(&mut self, event: InputEvent, host: &mut dyn Presenter) -> Flow;

    /// Something woke the loop: an engine event, a config reload or a timer.
    fn on_wake(&mut self, host: &mut dyn Presenter) -> Flow;

    /// The host wants a frame now (first paint, exposure, or a requested redraw).
    fn on_redraw(&mut self, host: &mut dyn Presenter);
}

pub trait Presenter {
    /// What this host can do; read at start and again after each callback
    /// that may have changed it.
    fn capabilities(&self) -> Capabilities;

    /// Shows a frame, repainting only what changed since the previous one.
    fn present(&mut self, frame: &Frame);

    /// Puts text on the system clipboard; a host without one ignores it.
    fn set_clipboard(&mut self, _text: &str) {}

    /// Sets the title of the window; a host without one ignores it.
    fn set_title(&mut self, _title: &str) {}

    /// Asks for a pointer shape; a host without shapes ignores it.
    fn set_pointer_shape(&mut self, shape: PointerShape);

    /// Asks the host to call `EventSink::on_redraw` soon.
    fn request_redraw(&mut self);

    /// A handle that wakes this host's loop from any thread.
    fn waker(&self) -> Waker;

    /// Runs the host's loop on the calling thread until the sink returns
    /// `Flow::Quit` or the host fails.
    fn run(&mut self, sink: &mut dyn EventSink) -> Result<(), HostError>;
}
