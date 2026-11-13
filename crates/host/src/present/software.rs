//! The canvas into the window, on the processor.
//!
//! No graphics card and no driver: the pixels are copied into a buffer the
//! window system shows. A card-backed presenter would be another file here
//! and is never the default.

use std::num::NonZeroU32;
use std::sync::Arc;

use surface::HostError;
use winit::window::Window;

use crate::paint::Canvas;

pub struct SoftwarePresenter {
    window: Arc<Window>,
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    /// Kept alive because the surface was made from it.
    _context: softbuffer::Context<Arc<Window>>,
}

impl SoftwarePresenter {
    pub fn new(window: Arc<Window>) -> Result<Self, HostError> {
        let context = softbuffer::Context::new(window.clone())
            .map_err(|error| HostError(format!("the window surface failed: {error}")))?;
        let surface = softbuffer::Surface::new(&context, window.clone())
            .map_err(|error| HostError(format!("the window surface failed: {error}")))?;
        Ok(Self {
            window,
            surface,
            _context: context,
        })
    }

    /// Copies the canvas into the window. A canvas that does not match the
    /// window's size is ignored, because the next redraw carries the right one.
    pub fn show(&mut self, canvas: &Canvas) -> Result<(), HostError> {
        let size = self.window.inner_size();
        let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            return Ok(());
        };
        if canvas.width() != size.width || canvas.height() != size.height {
            return Ok(());
        }
        self.surface
            .resize(width, height)
            .map_err(|error| HostError(format!("the window surface failed: {error}")))?;
        let mut buffer = self
            .surface
            .buffer_mut()
            .map_err(|error| HostError(format!("the window surface failed: {error}")))?;
        buffer.copy_from_slice(canvas.pixels());
        buffer
            .present()
            .map_err(|error| HostError(format!("showing the window failed: {error}")))
    }
}
