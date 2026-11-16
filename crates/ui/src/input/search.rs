//! Searching from the screen.
//!
//! `/` opens a box for the query; Enter asks the backend, and the answer
//! replaces the list until Escape brings the folder back. Typed letters are
//! the query, so no command key runs while the box is open. Until the answer
//! arrives the rows, and their title, are still the folder's. A search is
//! forgotten when the user goes to another folder, when its request fails, or
//! once its hits show and the folder's own page arrives, which is what a
//! delete or a move inside the results brings.

use model::{Command, Event, RequestId};
use surface::{InputEvent, KeyEvent, LogicalKey, NamedKey};

use super::line_edit::edit_line;
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
    pub(crate) fn search_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        match event {
            InputEvent::Key(key) => self.search_key(key),
            InputEvent::Text(text) | InputEvent::Paste(text) => {
                let clean: String = text.chars().filter(|ch| !ch.is_control()).collect();
                self.search_bar.as_mut().is_some_and(|bar| bar.insert_text(&clean))
            }
            _ => self.search_pointer(event, grid),
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
            other => return edit_line(bar, other, held),
        }
        true
    }

    /// Asks for the search; an empty query ends the one showing instead.
    pub(crate) fn run_search(&mut self) {
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
        // Hits already showing stay the rows, and keep their title, until the
        // answer to this search replaces them.
        let (showing, incomplete) =
            self.search.take().map_or((None, None), |old| (old.showing, old.incomplete));
        self.search = Some(ActiveSearch {
            query: query.clone(),
            request,
            incomplete,
            showing,
        });
        self.queue.push(Command::Search {
            request,
            account,
            query,
        });
    }

    /// Goes back from the results to the folder. A search still waiting for
    /// its answer has changed nothing on screen: the rows are the folder's.
    pub(crate) fn end_search(&mut self) {
        let Some(search) = self.search.take() else {
            return;
        };
        if search.showing.is_none() {
            return;
        }
        self.clear_marks();
        self.leave_hits();
        if let Some(folder) = self.selection.folder {
            self.queue.push(Command::OpenFolder { folder });
        }
    }

    /// The hits are not the folder's own list: drop them, so the folder's
    /// newest page replaces them instead of being merged with them (which
    /// would keep hits from other folders under this folder's name). The row
    /// that is selected goes back to the top, for the page to move it if the
    /// message is on it.
    fn leave_hits(&mut self) {
        self.mail.page = None;
        self.selection.message_index = 0;
    }

    /// The answer to the search that is showing replaces the list; an answer
    /// to an earlier one, or to none, is ignored. A fresh page of the folder
    /// that the screen takes ends a search whose hits are showing, and a
    /// failure of the search ends one still waiting for its answer.
    pub(crate) fn note_search_event(&mut self, event: &Event) {
        match event {
            Event::SearchResults { request, total, envelopes, incomplete } => {
                let current = self.search.as_ref().is_some_and(|search| search.request == *request);
                let Some(folder) = self.selection.folder.filter(|_| current) else {
                    return;
                };
                if let Some(search) = self.search.as_mut() {
                    search.incomplete.clone_from(incomplete);
                    search.showing = Some(search.query.clone());
                }
                // The footer says what was missed, so a partial answer is
                // not taken for the whole one.
                self.mail.set_note(incomplete.clone());
                self.mail.page = Some(ListPage {
                    folder,
                    offset: 0,
                    total: *total,
                    envelopes: envelopes.clone(),
                });
                self.selection.message = envelopes.first().map(|envelope| envelope.id);
                self.selection.message_index = 0;
                self.scroll.reset_list();
                self.scroll.reader = 0;
            }
            Event::EnvelopePage { folder, .. } => {
                let showing = self.search.as_ref().is_some_and(|search| search.showing.is_some());
                if showing && self.mail.accepts_page(*folder, &self.selection) {
                    self.search = None;
                    self.leave_hits();
                }
            }
            Event::OpFailed { account, operation, .. } if operation == "search" => {
                let ours = account.as_ref().is_none_or(|id| self.selection.account.as_ref() == Some(id));
                if ours && self.search.as_ref().is_some_and(|search| search.showing.is_none()) {
                    self.search = None;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
include!("search.tests.rs");
