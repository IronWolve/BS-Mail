//! Drawing what lies over the reading pane's text: the light on the link under
//! the pointer, the selected words, the matches of the find, and the find box
//! itself.

use surface::{Frame, Rect};

use crate::app_view::AppView;
use crate::input::link_light::LitLink;
use crate::widgets::find_bar::draw_find_bar;
use crate::widgets::{BodyView, DrawContext, Highlights, body_text, draw_highlights, draw_link_light};

impl AppView {
    /// Lights the link under the pointer on the body just drawn in `body` with
    /// `view`, then marks the selection and the matches over it (they win, and
    /// the light leaves the cells they cover alone), then draws the find box
    /// over its last row.
    pub(crate) fn draw_reading_marks(&self, frame: &mut Frame, ctx: &DrawContext<'_>, body: Rect, view: &BodyView<'_>, lit: Option<&LitLink>) {
        if lit.is_none() && self.reading.selection.is_none() && self.reading.find.is_none() {
            return;
        }
        let text = body_text(ctx, body, view);
        let matches = self.reading.find.as_ref().map(|find| find.matches(&text)).unwrap_or_default();
        let current = self.reading.find.as_ref().filter(|_| !matches.is_empty()).map(|find| find.current.min(matches.len() - 1));
        let marks = Highlights {
            selection: self.reading.selection.as_ref().and_then(|held| held.range(text.key())),
            found: &matches,
            current,
        };
        if let Some(lit) = lit {
            draw_link_light(frame, ctx, &text, body, (lit.hit.index, lit.state), &marks);
        }
        draw_highlights(frame, ctx, body, view, &marks);
        if let Some(find) = &self.reading.find {
            draw_find_bar(frame, ctx, body, &find.editor, &find.status(&matches), find.editing);
        }
    }
}
