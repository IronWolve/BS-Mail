//! Finding words in the open message. With the reader focused, `/` opens the
//! find box over the bottom of the pane (from the list `/` still searches the
//! account). Typing finds as it goes and the matches are marked; Enter goes
//! to the next match and Shift+Enter to the previous one, and then the box
//! stays with its result: `n` and `N` go to the next and the previous match,
//! `/` edits the query again and Escape closes the box. A message that is
//! opened while the box is open is searched too.

use std::rc::Rc;

use surface::{InputEvent, KeyEvent, LogicalKey, NamedKey};

use super::line_edit::edit_line;
use crate::app_view::AppView;
use crate::layout::Layout;
use crate::reader_tools::FindState;
use crate::view_state::Focus;
use crate::widgets::Match;

impl AppView {
    /// Whether the find box is open, taking keys or not.
    pub(crate) fn find_is_open(&self) -> bool {
        self.reading.find.is_some()
    }

    /// Whether the find box takes the typed letters.
    pub(crate) fn find_is_editing(&self) -> bool {
        self.reading.find.as_ref().is_some_and(|find| find.editing)
    }

    /// The find box is open and the reader has focus, which is when `n`, `N`,
    /// Enter and Escape are for it.
    pub(crate) fn find_has_the_reader(&self) -> bool {
        self.find_is_open() && self.focus == Focus::Reader
    }

    /// `/` in the reader: opens the box, or goes back into it to change the query.
    pub(crate) fn start_find(&mut self) {
        match self.reading.find.as_mut() {
            Some(find) => {
                find.editing = true;
                find.editor.select_all();
            }
            None => self.reading.find = Some(FindState::new("")),
        }
    }

    /// The menu's "Find in message": the reader takes focus and the box opens
    /// (when there is a message to look in).
    pub(crate) fn find_in_message(&mut self) {
        if self.reader_content().is_some() {
            self.focus = Focus::Reader;
            self.start_find();
        }
    }

    pub(crate) fn close_find(&mut self) {
        self.reading.find = None;
    }

    /// The matches of the find in what the pane shows, with the pane's text.
    fn found_in_view(&self, layout: &Layout) -> Option<(Rc<Vec<Match>>, usize, usize)> {
        let find = self.reading.find.as_ref()?;
        let (text, _) = self.reader_text(layout)?;
        Some((find.matches(&text), text.first(), text.visible()))
    }

    /// Makes the match at or after the top of the pane the current one, and
    /// shows it. Called when the query changes.
    fn find_from_top(&mut self, layout: &Layout) {
        let Some((matches, first, _)) = self.found_in_view(layout) else {
            return;
        };
        let at = matches.iter().position(|found| found.row >= first).unwrap_or(0);
        if let Some(find) = self.reading.find.as_mut() {
            find.current = at;
        }
        if let Some(found) = matches.get(at) {
            self.show_row(found.row, layout);
        }
    }

    /// Goes to the next match, or the previous one, wrapping round the ends.
    pub(crate) fn find_step(&mut self, forward: bool, grid: (u16, u16)) {
        let layout = self.layout(grid.0, grid.1);
        let Some((matches, ..)) = self.found_in_view(&layout) else {
            return;
        };
        let count = matches.len();
        let Some(find) = self.reading.find.as_mut().filter(|_| count > 0) else {
            return;
        };
        let now = find.current.min(count - 1);
        find.current = if forward { (now + 1) % count } else { (now + count - 1) % count };
        let row = matches[find.current].row;
        self.show_row(row, &layout);
    }

    /// Input while the box takes the typed letters. Returns whether the screen changed.
    pub(crate) fn find_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        match event {
            InputEvent::Key(key) => self.find_key(key, grid),
            InputEvent::Text(text) | InputEvent::Paste(text) => {
                let clean: String = text.chars().filter(|ch| !ch.is_control()).collect();
                let changed = self.reading.find.as_mut().is_some_and(|find| find.editor.insert_text(&clean));
                if changed {
                    let layout = self.layout(grid.0, grid.1);
                    self.find_from_top(&layout);
                }
                changed
            }
            _ => self.find_pointer(event, grid),
        }
    }

    fn find_key(&mut self, key: &KeyEvent, grid: (u16, u16)) -> bool {
        let held = key.modifiers;
        let LogicalKey::Named(named) = key.key else {
            return false;
        };
        match named {
            NamedKey::Escape => self.close_find(),
            NamedKey::Enter => {
                if let Some(find) = self.reading.find.as_mut() {
                    find.editing = false;
                }
                self.find_step(!held.shift, grid);
            }
            other => {
                let Some(find) = self.reading.find.as_mut() else {
                    return false;
                };
                let before = find.query();
                if !edit_line(&mut find.editor, other, held) {
                    return false;
                }
                if find.query() != before {
                    let layout = self.layout(grid.0, grid.1);
                    self.find_from_top(&layout);
                }
            }
        }
        true
    }
}

#[cfg(test)]
include!("find_keys.tests.rs");
