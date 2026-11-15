//! Drawing the list and the reader. Both draw only the page of the folder that
//! is selected: while another folder's page is still held (the new folder's
//! has not arrived), the panes are empty and no row or header of the old
//! folder is shown under the new folder's name.

use surface::Frame;

use crate::app_view::AppView;
use crate::input::viewport::list_first;
use crate::layout::{ListRegion, ReaderRegion};
use crate::options::empty_hint::draw_no_account_hint;
use crate::view_state::ListPage;
use crate::widgets::{
    BodyView, DrawContext, HighlightRules, ListRowsView, ReaderTitleData, draw_list_header,
    draw_list_rows, draw_list_title, draw_reader_body, draw_reader_headers, draw_reader_title,
};

impl AppView {
    /// The envelope of the selected message, from the page of the selected
    /// folder.
    fn selected_envelope(&self) -> Option<&model::Envelope> {
        self.shown_page()?.envelopes.get(self.selection.message_index)
    }

    pub(crate) fn draw_list(&self, frame: &mut Frame, ctx: &DrawContext<'_>, list: &ListRegion) {
        let mailbox = self.list_summary();
        draw_list_title(frame, ctx, list.title, mailbox.as_ref());
        draw_list_header(frame, ctx, list.header, &list.columns);
        if self.options_host.has_no_account() {
            draw_no_account_hint(frame, ctx, list.rows);
            return;
        }
        let Some(page) = self.shown_page() else {
            return;
        };
        let visible = usize::from(list.rows.height);
        let selected = self.selection.message_index;
        let view = ListRowsView {
            envelopes: &page.envelopes,
            columns: &list.columns,
            selected: Some(selected),
            offset: page.offset,
            first: list_first(self.scroll.list, selected, visible, page.envelopes.len()),
            now: self.now,
        };
        draw_list_rows(frame, ctx, list.rows, &view);
        self.show_marks(frame, list, view.first, &page.envelopes);
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
        }
    }
}
