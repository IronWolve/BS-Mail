//! The keys of the reading pane, as one table: copy, select, find, the raw
//! headers and source, and the browser. Each only calls the feature's own
//! file. What `n`, `N`, Enter and Escape mean while the find box is open, and
//! what Escape closes first in the reader, is decided here.

use model::RawPart;

use super::action::Action;
use crate::app_view::AppView;
use crate::view_state::Focus;

impl AppView {
    /// Does `action` when it is one of the reading pane's; false for any
    /// other, which the caller then handles.
    pub(super) fn act_on_reading(&mut self, action: Action, grid: (u16, u16)) -> bool {
        match action {
            Action::Copy => self.copy_key(grid),
            Action::CopySender => self.copy_sender(),
            Action::ViewHeaders => self.view_raw(RawPart::Headers),
            Action::ViewSource => self.view_raw(RawPart::Source),
            Action::OpenInBrowser => self.open_in_browser(),
            Action::NextAttachment => self.step_attachment(true),
            Action::PreviousAttachment => self.step_attachment(false),
            Action::Extend(direction) if self.focus == Focus::Reader => self.extend_selection(direction, grid),
            Action::Extend(direction) => self.act(direction.plain(), grid),
            Action::Search if self.focus == Focus::Reader && self.reader_content().is_some() => self.start_find(),
            Action::NextMessage | Action::Enter if self.find_has_the_reader() => self.find_step(true, grid),
            Action::NextUnread if self.find_has_the_reader() => self.find_step(false, grid),
            Action::Back if self.focus == Focus::Reader && self.close_one_reader_extra() => {}
            _ => return false,
        }
        true
    }

    /// Escape in the reader closes what is on top: the find box, then the raw
    /// view, then the selection. Returns whether it closed anything.
    fn close_one_reader_extra(&mut self) -> bool {
        if self.find_is_open() {
            self.close_find();
        } else if self.reading.raw_of(self.selection.message).is_some() {
            self.reading.raw = None;
            self.scroll.reader = 0;
        } else if self.reading.selection.is_some() {
            self.reading.selection = None;
        } else {
            return false;
        }
        true
    }
}
