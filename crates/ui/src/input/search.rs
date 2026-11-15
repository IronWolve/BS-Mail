//! Searching from the screen.
//!
//! `/` opens a box for the query; Enter asks the backend, and the answer
//! replaces the list until Escape brings the folder back. Typed letters are
//! the query, so no command key runs while the box is open. A search is
//! forgotten when the user goes to another folder, or when the folder's own
//! page arrives, which is what a delete or a move inside the results brings.

use model::{Command, Event, RequestId};
use surface::{InputEvent, KeyEvent, LogicalKey, NamedKey};

use crate::app_view::AppView;
use crate::composer::EditorBuffer;
use crate::search::ActiveSearch;
use crate::view_state::ListPage;

impl AppView {
    /// Opens the box, with the last query in it to be changed.
    pub(crate) fn start_search(&mut self) {
        if self.selected_account().is_none() {
            return;
        }
        let last = self.search.as_ref().map_or("", |search| search.query.as_str());
        self.search_bar = Some(EditorBuffer::single_line(last));
    }

    /// Input while the box is open. Returns whether the screen changed.
    pub(crate) fn search_input(&mut self, event: &InputEvent) -> bool {
        match event {
            InputEvent::Key(key) => self.search_key(key),
            InputEvent::Text(text) | InputEvent::Paste(text) => {
                let clean: String = text.chars().filter(|ch| !ch.is_control()).collect();
                self.search_bar.as_mut().is_some_and(|bar| bar.insert_text(&clean))
            }
            _ => false,
        }
    }

    fn search_key(&mut self, key: &KeyEvent) -> bool {
        let held = key.modifiers;
        let LogicalKey::Named(named) = key.key else {
            return false;
        };
        let Some(bar) = self.search_bar.as_mut() else {
            return false;
        };
        match named {
            NamedKey::Escape => self.search_bar = None,
            NamedKey::Enter => self.run_search(),
            NamedKey::Left if held.ctrl => bar.move_word_left(held.shift),
            NamedKey::Right if held.ctrl => bar.move_word_right(held.shift),
            NamedKey::Left => bar.move_left(held.shift),
            NamedKey::Right => bar.move_right(held.shift),
            NamedKey::Home => bar.move_line_start(held.shift),
            NamedKey::End => bar.move_line_end(held.shift),
            NamedKey::Backspace if held.ctrl => {
                bar.delete_word_back();
            }
            NamedKey::Backspace => {
                bar.backspace();
            }
            NamedKey::Delete => {
                bar.delete_forward();
            }
            _ => return false,
        }
        true
    }

    /// Asks for the search; an empty query ends the one showing instead.
    fn run_search(&mut self) {
        let Some(bar) = self.search_bar.take() else {
            return;
        };
        let query = bar.text().trim().to_string();
        if query.is_empty() {
            self.end_search();
            return;
        }
        let Some(account) = self.selection.account.clone() else {
            return;
        };
        self.next_request += 1;
        let request = RequestId(self.next_request);
        self.search = Some(ActiveSearch {
            query: query.clone(),
            request,
            incomplete: None,
        });
        self.queue.push(Command::Search {
            request,
            account,
            query,
        });
    }

    /// Goes back from the results to the folder.
    pub(crate) fn end_search(&mut self) {
        if self.search.take().is_none() {
            return;
        }
        self.clear_marks();
        if let Some(folder) = self.selection.folder {
            self.queue.push(Command::OpenFolder { folder });
        }
    }

    /// The answer to the search that is showing replaces the list; an answer
    /// to an earlier one, or to none, is ignored. A fresh page of the folder
    /// ends the search.
    pub(crate) fn note_search_event(&mut self, event: &Event) {
        match event {
            Event::SearchResults { request, total, envelopes, incomplete } => {
                let current = self.search.as_ref().is_some_and(|search| search.request == *request);
                let Some(folder) = self.selection.folder.filter(|_| current) else {
                    return;
                };
                if let Some(search) = self.search.as_mut() {
                    search.incomplete.clone_from(incomplete);
                }
                // The footer says what was missed, so a partial answer is
                // not taken for the whole one.
                self.mail.error = incomplete.clone();
                self.mail.page = Some(ListPage {
                    folder,
                    offset: 0,
                    total: *total,
                    envelopes: envelopes.clone(),
                });
                self.selection.message = envelopes.first().map(|envelope| envelope.id);
                self.selection.message_index = 0;
                self.scroll.list = 0;
                self.scroll.reader = 0;
            }
            Event::EnvelopePage { .. } => self.search = None,
            _ => {}
        }
    }
}

#[cfg(test)]
include!("search.tests.rs");
