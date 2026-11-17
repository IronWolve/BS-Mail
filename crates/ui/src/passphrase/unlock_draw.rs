//! Drawing the start-up prompt on the whole grid.

use surface::{Frame, Rect};

use super::frame::draw_prompt;
use super::layout::{FieldId, place};
use super::unlock::UnlockView;
use crate::format::display_width;
use crate::theme::roles;
use crate::widgets::{DrawContext, fill_cells, put_text};
use model::MailDate;

const TOO_SMALL: &str = "Passphrase needed: make the window larger";

impl UnlockView {
    /// The prompt for a `cols` x `rows` grid.
    pub fn render(&self, cols: u16, rows: u16) -> Frame {
        let area = Rect::new(0, 0, cols, rows);
        let mut frame = Frame::new(area);
        frame.line_weight = self.config.appearance.line_weight.max(1);
        let ctx = DrawContext::new(&self.theme, &self.icons, &self.config, MailDate::default());
        fill_cells(&mut frame, area, ctx.full(roles::BASE));
        let prompt = self.prompt();
        let Some(placed) = place(area, &prompt) else {
            let text = if display_width(TOO_SMALL) <= usize::from(cols.saturating_sub(1)) { TOO_SMALL } else { "make it larger" };
            put_text(&mut frame, area, area.x, area.y, text, ctx.fg(roles::FOOTER_ERROR));
            return frame;
        };
        draw_prompt(&mut frame, &ctx, &prompt, &placed, self.focus, &[(FieldId::First, self.line.chars())]);
        frame
    }
}

#[cfg(test)]
include!("unlock_draw.tests.rs");
