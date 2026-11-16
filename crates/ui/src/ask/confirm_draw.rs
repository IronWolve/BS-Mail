//! A folder question on screen: the same box as the question before a message
//! is deleted (see `delete_prompt::draw`), with this question's words.

use surface::{Frame, Rect};

use super::confirm::ConfirmPrompt;
use crate::delete_prompt::draw::{BoxText, draw_box};
use crate::widgets::DrawContext;

pub fn draw_confirm(frame: &mut Frame, ctx: &DrawContext<'_>, grid: Rect, prompt: &ConfirmPrompt) {
    let text = BoxText {
        title: prompt.title,
        question: &prompt.question,
        warning: prompt.warning,
        confirm: prompt.yes,
        keep: prompt.no,
        focus: prompt.focus,
    };
    draw_box(frame, ctx, grid, &text);
}

#[cfg(test)]
include!("confirm_draw.tests.rs");
