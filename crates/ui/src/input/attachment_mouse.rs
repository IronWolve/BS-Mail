//! The mouse on the attachment lines of the open message: a click (or a right
//! click) on one puts the cursor on it and opens its menu, Save, Save as... and
//! Open, at the pointer. A line that was wrapped is one attachment whichever of
//! its rows is clicked.

use super::menu_tables::MenuKind;
use super::menus::MenuTarget;
use crate::app_view::AppView;
use crate::layout::Layout;
use crate::view_state::Focus;
use crate::widgets::{BodyView, DrawContext, HighlightRules, body_pos_at, body_text};

impl AppView {
    /// The attachment whose line is at the cell `col`, `row` of the reader's
    /// body, as its place in the list.
    pub(crate) fn attachment_at(&self, layout: &Layout, col: u16, row: u16) -> Option<usize> {
        let count = self.reader_attachments().len();
        if count == 0 {
            return None;
        }
        let reader = layout.reader.as_ref()?;
        let content = self.reader_content()?;
        let ctx = DrawContext::new(&self.theme, &self.icons, &self.config, self.now);
        let view = BodyView {
            lead: &content.lead,
            lines: content.body,
            first: self.scroll.reader,
            app_name: &self.identity.name,
            rules: HighlightRules::default(),
        };
        let pos = body_pos_at(&ctx, reader.body, &view, col, row)?;
        let line = body_text(&ctx, reader.body, &view).line_of_row(pos.row);
        (line < count).then_some(line)
    }

    /// Puts the cursor on attachment `at` and opens its menu at `anchor`.
    pub(crate) fn open_attachment_menu(&mut self, at: usize, anchor: (u16, u16)) {
        let Some(message) = self.selection.message else {
            return;
        };
        self.reading.attachments.cursor = Some((message, at));
        self.reading.link_cursor = None;
        self.focus = Focus::Reader;
        self.open_menu(MenuKind::Attachment, MenuTarget::Attachment(at), anchor);
    }

    /// A press on an attachment line of the reader: its menu opens. False when
    /// the press is not on one.
    pub(crate) fn click_attachment(&mut self, layout: &Layout, (col, row): (u16, u16)) -> bool {
        let Some(at) = self.attachment_at(layout, col, row) else {
            return false;
        };
        self.open_attachment_menu(at, (col, row));
        true
    }
}

#[cfg(test)]
include!("attachment_mouse.tests.rs");
