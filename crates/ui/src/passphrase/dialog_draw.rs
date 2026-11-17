//! Drawing the dialog over the Options screen.

use surface::{Frame, Rect};

use super::dialog::PassphraseDialog;
use super::frame::draw_prompt;
use super::layout::{FieldId, place};
use crate::format::display_width;
use crate::theme::roles;
use crate::widgets::{DrawContext, put_text};

const TOO_SMALL: &str = "Make the window larger to continue (Esc closes this)";

/// Draws the dialog in `area`, over what is already drawn.
pub fn draw_dialog(frame: &mut Frame, ctx: &DrawContext<'_>, area: Rect, dialog: &PassphraseDialog) {
    let prompt = dialog.prompt();
    let Some(placed) = place(area, &prompt) else {
        let text = if display_width(TOO_SMALL) <= usize::from(area.width) { TOO_SMALL } else { "make it larger" };
        put_text(frame, area, area.x + 1, area.y, text, ctx.fg(roles::FOOTER_ERROR));
        return;
    };
    let (first, second) = dialog.typed_chars();
    draw_prompt(frame, ctx, &prompt, &placed, dialog.focus, &[(FieldId::First, first), (FieldId::Second, second)]);
}

#[cfg(test)]
include!("dialog_draw.tests.rs");
