//! The open message's text as the reading pane draws it, for the keys and the
//! pointer that work on what is shown: the wrapped rows, a place under the
//! pointer, and the message's own lines.

use surface::Rect;

use crate::app_view::AppView;
use crate::layout::Layout;
use crate::widgets::{BodyPos, BodyText, BodyView, DrawContext, HighlightRules, body_pos_at, body_text};

impl AppView {
    /// The wrapped rows of what the reading pane shows, and the rectangle its
    /// text is drawn in. `None` while there is nothing to read.
    pub(crate) fn reader_text(&self, layout: &Layout) -> Option<(BodyText, Rect)> {
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
        Some((body_text(&ctx, reader.body, &view), reader.body))
    }

    /// The place in the text under a cell of the screen.
    pub(crate) fn reader_pos_at(&self, layout: &Layout, col: u16, row: u16) -> Option<BodyPos> {
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
        body_pos_at(&ctx, reader.body, &view, col, row)
    }

    /// Every line of the message (or of the raw view), as the sender wrote
    /// them, with no attachment lines and no wrapping.
    pub(crate) fn whole_text(&self) -> Option<String> {
        let content = self.reader_content()?;
        let lines: Vec<String> =
            content.body.iter().map(|line| line.spans.iter().map(|span| span.text.as_str()).collect()).collect();
        Some(lines.join("\n"))
    }

    /// Scrolls the reading pane so that `row` of the wrapped text is on
    /// screen, a third of the way down when it has to move.
    pub(crate) fn show_row(&mut self, row: usize, layout: &Layout) {
        let Some((lines, height)) = self.reader_extent(layout) else {
            return;
        };
        let first = self.scroll.reader;
        if row < first || row >= first + height {
            self.scroll.reader = row.saturating_sub(height / 3).min(lines.saturating_sub(height));
        }
    }
}
