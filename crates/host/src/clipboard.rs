//! The system clipboard, for copy and paste between this program and others.
//!
//! The clipboard is opened the first time it is needed, not at start: a
//! machine with no clipboard service, or no display, then costs nothing and
//! fails only when someone actually tries to copy or paste.

pub struct SystemClipboard {
    inner: Option<arboard::Clipboard>,
}

impl SystemClipboard {
    pub fn new() -> Self {
        Self { inner: None }
    }

    fn open(&mut self) -> Option<&mut arboard::Clipboard> {
        if self.inner.is_none() {
            self.inner = arboard::Clipboard::new().ok();
        }
        self.inner.as_mut()
    }

    /// The text on the clipboard, or `None` when there is none or it cannot
    /// be read.
    pub fn text(&mut self) -> Option<String> {
        let text = self.open()?.get_text().ok()?;
        (!text.is_empty()).then_some(text)
    }

    /// The text of the primary selection (what was last selected, pasted with
    /// the middle button), where the system has one: X11, and Wayland when the
    /// compositor offers it. `None` when there is none, it cannot be read, or
    /// the system has no such selection (Windows and macOS).
    #[cfg(target_os = "linux")]
    pub fn primary_text(&mut self) -> Option<String> {
        use arboard::{GetExtLinux, LinuxClipboardKind};
        let text = self.open()?.get().clipboard(LinuxClipboardKind::Primary).text().ok()?;
        (!text.is_empty()).then_some(text)
    }

    #[cfg(not(target_os = "linux"))]
    pub fn primary_text(&mut self) -> Option<String> {
        None
    }

    /// Puts text on the clipboard. Returns whether it worked.
    pub fn set_text(&mut self, text: &str) -> bool {
        self.open().is_some_and(|clipboard| clipboard.set_text(text.to_string()).is_ok())
    }
}

impl Default for SystemClipboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
include!("clipboard.tests.rs");
