//! Drawing the list and the reader. Both draw only the page of the folder that
//! is selected: while another folder's page is still held (the new folder's
//! has not arrived), the panes are empty and no row or header of the old
//! folder is shown under the new folder's name.

use surface::Frame;

use crate::app_view::AppView;
use crate::layout::{ListRegion, ReaderRegion};
use crate::options::empty_hint::draw_no_account_hint;
use crate::view_state::ListPage;
use crate::widgets::{
    BodyView, DrawContext, HighlightRules, ListRowsView, ReaderTitleData, column_label_for, draw_list_header,
    draw_list_rows, draw_list_title, draw_scrollbar, draw_reader_body, draw_reader_headers, draw_reader_title,
};

impl AppView {
    /// The envelope of the selected message, from the page of the selected
    /// folder.
    fn selected_envelope(&self) -> Option<&model::Envelope> {
        self.shown_page()?.envelopes.get(self.selection.message_index)
    }

    pub(crate) fn draw_list(&self, frame: &mut Frame, ctx: &DrawContext<'_>, list: &ListRegion) {
        let mailbox = self.list_summary();
        draw_list_title(frame, ctx, list.title, mailbox.as_ref(), self.order_note().as_deref());
        let page = self.shown_page();
        let from_label = self.from_column_label(page);
        draw_list_header(frame, ctx, list.header, &list.columns, from_label, self.list_sort());
        if self.options_host.has_no_account() {
            draw_no_account_hint(frame, ctx, list.rows);
            return;
        }
        let Some(page) = page else {
            return;
        };
        let visible = usize::from(list.rows.height);
        let selected = self.selection.message_index;
        let view = ListRowsView {
            envelopes: &page.envelopes,
            rows: &page.rows,
            folders: &self.mail.folders,
            columns: &list.columns,
            selected: Some(selected),
            offset: page.offset,
            first: self.list_view_first(visible, page.envelopes.len()),
            now: self.now,
        };
        draw_list_rows(frame, ctx, list.rows, &view);
        if let Some(bar) = self.list_bar(list) {
            draw_scrollbar(frame, ctx, bar.col, bar.rows, bar.first, bar.visible, bar.total);
        }
        self.show_marks(frame, list, view.first, page);
    }

    /// What the sender column is called over the rows on screen: `To` in the
    /// Sent and Drafts folders, `From` everywhere else.
    pub(crate) fn from_column_label(&self, page: Option<&ListPage>) -> &'static str {
        // A search list is not the folder's own, so an empty one says nothing
        // about whom the rows would name.
        let own_list = self.search.is_none().then_some(self.selection.folder).flatten();
        let folder_role = own_list.and_then(|id| self.mail.folder(id)).map(|folder| folder.role);
        column_label_for(&self.mail.folders, page.map_or(&[], |page| page.envelopes.as_slice()), folder_role)
    }

    fn reader_title_data<'a>(&'a self, page: &'a ListPage) -> Option<ReaderTitleData<'a>> {
        let envelope = page.envelopes.get(self.selection.message_index)?;
        let unread = self.list_summary().map_or(0, |mailbox| mailbox.unread);
        Some(ReaderTitleData {
            subject: &envelope.subject,
            position: u64::from(page.offset) + self.selection.message_index as u64 + 1,
            total: u64::from(page.total),
            unread,
        })
    }

    pub(crate) fn draw_reader(&self, frame: &mut Frame, ctx: &DrawContext<'_>, reader: &ReaderRegion) {
        let title = self.shown_page().and_then(|page| self.reader_title_data(page));
        draw_reader_title(frame, ctx, reader.title, title.as_ref());
        draw_reader_headers(frame, ctx, reader.headers, self.selected_envelope());
        if let Some(content) = self.reader_content() {
            let view = BodyView {
                lead: &content.lead,
                lines: content.body,
                first: self.scroll.reader,
                app_name: &self.identity.name,
                rules: HighlightRules::default(),
            };
            draw_reader_body(frame, ctx, reader.body, &view);
            self.draw_reading_marks(frame, ctx, reader.body, &view);
        }
    }
}

#[cfg(test)]
include!("view_panes.tests.rs");
