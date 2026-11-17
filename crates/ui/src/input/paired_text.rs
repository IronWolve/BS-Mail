//! The text that comes with a command key.
//!
//! A window and a terminal both report a printable key as the key and then the
//! text it types. On the main screen the key is a command, and what it opens
//! (the composer, the search box, the move picker) must not also be handed
//! the letter that opened it: `c` for a new message would otherwise start the
//! address with a `c`.

use surface::{InputEvent, LogicalKey};

use crate::app_view::AppView;

/// A printable key with no Ctrl, Alt or Meta, the kind a terminal and a window
/// both follow with the text it types.
fn command_letter(event: &InputEvent) -> bool {
    match event {
        InputEvent::Key(key) => {
            matches!(key.key, LogicalKey::Char(ch) if !ch.is_control())
                && !key.modifiers.ctrl
                && !key.modifiers.alt
                && !key.modifiers.meta
        }
        _ => false,
    }
}

impl AppView {
    /// Reacts to one input event on a `cols` x `rows` grid. Returns whether
    /// anything on screen may have changed, so the host knows to redraw.
    pub fn handle_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        // The notice of new mail stays only until the next key.
        let cleared = matches!(event, InputEvent::Key(_)) && self.new_mail.take().is_some();
        // Only a move of the pointer leaves what it found under it as it was.
        if !matches!(event, InputEvent::PointerMoved { .. }) {
            self.reading.link_answers.forget();
        }
        let lit = self.track_hover(event, grid);
        self.handle_unnoticed_input(event, grid) || cleared || lit
    }

    fn handle_unnoticed_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        let paired = self.paired_text.take();
        if let (InputEvent::Text(text), Some(letter)) = (event, paired)
            && *text == letter.to_string()
        {
            // The text of the command key that was just handled: it did its
            // work as a key, and typing it again into what it opened (the
            // composer, the search box) would put a stray letter there.
            return false;
        }
        let command_key = self.ignores_text() && command_letter(event);
        let used = self.dispatch_input(event, grid);
        if command_key && let InputEvent::Key(key) = event && let LogicalKey::Char(letter) = key.key {
            self.paired_text = Some(letter);
        }
        used
    }

    /// True while nothing open reads typed text: the main screen, a question
    /// or a help page. A letter pressed there is a command, whatever it opens.
    fn ignores_text(&self) -> bool {
        self.setup.is_none()
            && self.options.is_none()
            && self.passphrase_dialog.is_none()
            && self.composer.is_none()
            && self.picker.is_none()
            && self.ask.is_none()
            && self.search_bar.is_none()
            && !self.find_is_editing()
    }
}
