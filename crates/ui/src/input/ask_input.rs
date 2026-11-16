//! Input while one of the boxes of `ask` is open: it takes every key and
//! click, and nothing reaches the screen underneath. This file only routes to
//! the box that is open (`ask_name`, `ask_confirm`, `ask_tags`); the screen
//! too small to show a box can only dismiss it, never answer it.

use surface::InputEvent;

use super::question::refuses;
use crate::app_view::AppView;
use crate::ask::Ask;

impl AppView {
    /// Whether the open box has a line to type in, so a letter is for the box.
    pub(crate) fn ask_takes_text(&self) -> bool {
        matches!(self.ask, Some(Ask::Name(_) | Ask::Tag(_)))
    }

    /// Input while a box is open. Returns whether the screen changed.
    pub(crate) fn ask_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        if self.layout(grid.0, grid.1).too_small {
            // Only the line saying the window is too small is drawn, so the
            // box cannot be seen: it can be dismissed and never answered.
            return match event {
                InputEvent::Key(key) if refuses(key) => {
                    self.ask = None;
                    true
                }
                _ => false,
            };
        }
        match self.ask {
            Some(Ask::Name(_)) => self.name_input(event, grid),
            Some(Ask::Confirm(_)) => self.confirm_input(event, grid),
            Some(Ask::Tag(_)) => self.tag_input(event, grid),
            None => false,
        }
    }
}
