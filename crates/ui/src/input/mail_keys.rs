//! The keys that act on messages, as one table: read and star, delete, the
//! marks, the moves (picker, Archive, Spam) and stepping through the list.
//! Each only calls the feature's own file; none talks to the backend here.

use super::action::Action;
use crate::app_view::AppView;

impl AppView {
    /// Does `action` when it is one that acts on messages; false for any
    /// other, which the caller then handles.
    pub(super) fn act_on_messages(&mut self, action: Action, grid: (u16, u16)) -> bool {
        match action {
            Action::ToggleRead => self.toggle_read(),
            Action::ToggleStar => self.toggle_star(),
            Action::Delete => self.delete_selected(),
            Action::MarkAll => self.mark_all(),
            Action::UnmarkAll => self.clear_marks(),
            Action::InvertMarks => self.invert_marks(),
            Action::Move => self.start_move(),
            Action::Archive => self.archive_targets(),
            Action::Spam => self.spam_targets(),
            Action::NextMessage => self.show_neighbor(Action::Down, grid),
            Action::PreviousMessage => self.show_neighbor(Action::Up, grid),
            Action::NextUnread => self.show_next_unread(grid),
            _ => return false,
        }
        true
    }
}
