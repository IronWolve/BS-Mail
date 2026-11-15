//! Scrolling the help box on a grid too short to show every row.

use surface::{InputEvent, Rect};

use super::action::{Action, action_for};
use super::movement::{WHEEL_ROWS, step};
use crate::app_view::AppView;
use crate::view_state::OverlayKind;
use crate::widgets::help_overlay::{help_capacity, help_max_scroll, help_rect};
use crate::widgets::help_rows::help_rows;

impl AppView {
    /// Takes the keys and the wheel that scroll the help list when some of
    /// its rows are out of sight; returns true when it used the event. With
    /// every row visible there is nothing to scroll and any key closes help.
    pub(crate) fn help_scrolls(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        if self.overlays.open != Some(OverlayKind::Help) {
            return false;
        }
        let area = Rect::new(0, 0, grid.0, grid.1);
        let capabilities = self.capabilities();
        let max = help_max_scroll(area, &capabilities);
        if max == 0 {
            return false;
        }
        let page = help_capacity(help_rect(area, &help_rows(&capabilities)));
        let current = self.overlays.help_scroll.min(max);
        let next = match event {
            InputEvent::Key(key) => match action_for(key) {
                Some(action) if is_scroll(action) => step(action, current, max + 1, page),
                _ => return false,
            },
            InputEvent::Wheel { rows, .. } if *rows != 0 => {
                let action = if *rows < 0 { Action::Up } else { Action::Down };
                let notches = usize::from(rows.unsigned_abs()) * WHEEL_ROWS;
                (0..notches).fold(current, |at, _| step(action, at, max + 1, page))
            }
            _ => return false,
        };
        self.overlays.help_scroll = next;
        true
    }
}

fn is_scroll(action: Action) -> bool {
    matches!(
        action,
        Action::Up | Action::Down | Action::PageUp | Action::PageDown | Action::First | Action::Last
    )
}

#[cfg(test)]
include!("help_scroll.tests.rs");
