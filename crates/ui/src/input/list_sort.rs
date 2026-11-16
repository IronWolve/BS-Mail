//! Sorting the list: `o` goes to the next column (the folder's own order,
//! From or To, Subject, Size, Flag, Date and round again), `O` reverses the
//! order, and a click on a column's header sorts by it, or reverses it when
//! the list is sorted by it already. A click on the `#` header goes back to
//! the folder's own order.
//!
//! Only the loaded mail is sorted, and the list's title says so while older
//! mail is not loaded. The choice is kept for each kind of folder (see
//! `sort_prefs`) and is what the state file remembers. A sort never changes
//! what a key acts on: the selection stays on its message, and the marks stay
//! on theirs.

use crate::app_view::AppView;
use crate::layout::{ColumnId, Layout};
use crate::list_order::sort::{click_sort, next_sort, reverse_sort};
use crate::list_order::{Arrangement, RoleKey, Sort, SortColumn, SortPrefs};
use crate::view_state::Focus;
use crate::widgets::list_rows::party::TO_LABEL;

/// What the footer says when the sort key is pressed away from the list.
const FOCUS_NOTE: &str = "o sorts the list: press 2 to focus it first";

impl AppView {
    /// The kind of folder the list shows, for the sort kept for it.
    pub(crate) fn sort_role(&self) -> RoleKey {
        let role = self.selection.folder.and_then(|id| self.mail.folder(id)).map(|folder| folder.role).unwrap_or_default();
        RoleKey::of(role)
    }

    /// The sort the list has now: the user's choice for this kind of folder,
    /// else the setting. `None` is the folder's own order.
    pub fn list_sort(&self) -> Option<Sort> {
        self.mail.arrangement.sort_for(self.sort_role())
    }

    /// The sorts the user chose, for the state file.
    pub fn sort_prefs(&self) -> SortPrefs {
        self.mail.arrangement.prefs.clone()
    }

    /// Takes the sorts the state file kept. The list is made again if it is
    /// showing already.
    pub fn restore_sorts(&mut self, prefs: SortPrefs) {
        self.mail.arrangement.prefs = prefs;
        self.arrangement_changed();
    }

    /// Takes what `[list]` says about arranging the list, when the settings
    /// are read again: the sort for the kinds of folder nothing was chosen
    /// for, and whether threads are on. What the user chose stays.
    pub(crate) fn take_list_settings(&mut self, list: &model::config::ListConfig) {
        let fresh = Arrangement::from_config(list);
        self.mail.arrangement.default_sort = fresh.default_sort;
        self.mail.arrangement.threaded = fresh.threaded;
        self.arrangement_changed();
    }

    /// `o`: the next column.
    pub(crate) fn sort_next(&mut self) {
        if self.focus != Focus::List {
            self.mail.set_note(Some(FOCUS_NOTE.to_string()));
            return;
        }
        self.set_sort(next_sort(self.list_sort()));
    }

    /// `O`: the other way round.
    pub(crate) fn sort_reverse(&mut self) {
        if self.focus != Focus::List {
            self.mail.set_note(Some(FOCUS_NOTE.to_string()));
            return;
        }
        self.set_sort(reverse_sort(self.list_sort()));
    }

    /// A click on the header of `column`: this sort, or the other way round
    /// when the list is sorted by it already. `None` is the `#` column, which
    /// brings back the folder's own order.
    pub(crate) fn sort_by_header(&mut self, column: Option<SortColumn>) {
        let sort = match column {
            Some(column) => click_sort(self.list_sort(), column),
            None => None,
        };
        self.set_sort(sort);
    }

    /// Keeps `sort` for this kind of folder and makes the rows again.
    fn set_sort(&mut self, sort: Option<Sort>) {
        let role = self.sort_role();
        self.mail.arrangement.prefs.set(role, sort);
        self.arrangement_changed();
    }

    /// A press at (`col`, `row`) in the list's header: the sort of the column
    /// it is on. False when it is on no column.
    pub(crate) fn press_header(&mut self, layout: &Layout, col: u16, row: u16) -> bool {
        let Some(column) = self.header_column_at(layout, col, row) else {
            return false;
        };
        self.sort_by_header(sort_column_of(column));
        true
    }

    /// The column whose header lies at (`col`, `row`).
    pub(crate) fn header_column_at(&self, layout: &Layout, col: u16, row: u16) -> Option<ColumnId> {
        let list = layout.list.as_ref()?;
        if row != list.header.y || col < list.header.x || col >= list.header.right() {
            return None;
        }
        self.message_cell(layout, col)
    }

    /// What the list's title adds about the order: `threads, sorted by size
    /// (loaded mail only)`. The loaded-only note stays while older mail is
    /// not loaded. `None` while the list is in its own order with no threads.
    pub(crate) fn order_note(&self) -> Option<String> {
        let sort = self.list_sort();
        let threaded = self.mail.arrangement.threaded;
        if sort.is_none() && !threaded {
            return None;
        }
        let mut parts = Vec::new();
        if threaded {
            parts.push("threads".to_string());
        }
        if let Some(sort) = sort {
            let recipients = self.from_column_label(self.shown_page()) == TO_LABEL;
            parts.push(format!("sorted by {}", sort.column.title_name(recipients)));
        }
        let partial = self.shown_page().is_some_and(|page| !page.all_loaded());
        let tail = if partial { " (loaded mail only)" } else { "" };
        Some(format!("{}{tail}", parts.join(", ")))
    }
}

/// The sort column a column's header stands for; `#` stands for none.
pub(crate) fn sort_column_of(column: ColumnId) -> Option<SortColumn> {
    match column {
        ColumnId::Num => None,
        ColumnId::Flag | ColumnId::Star => Some(SortColumn::Flag),
        ColumnId::From => Some(SortColumn::From),
        ColumnId::Subject => Some(SortColumn::Subject),
        ColumnId::Date => Some(SortColumn::Date),
        ColumnId::Size => Some(SortColumn::Size),
    }
}

#[cfg(test)]
include!("list_sort.tests.rs");
