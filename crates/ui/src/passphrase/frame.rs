//! Drawing a placed passphrase prompt: the box with its title and key hint,
//! the lines of text, the fields (one dot for each character typed, never the
//! characters) and the buttons.

use surface::{CursorRequest, CursorShape, Frame, Modifier, Rect, Style};

use super::layout::{ButtonId, FieldId, Placed, Prompt, Tone};
use crate::format::display_width;
use crate::theme::roles;
use crate::widgets::help_overlay::overlaps;
use crate::widgets::{DrawContext, fill_cells, put_text};

/// What a dot stands for.
pub const DOT: &str = "\u{2022}";

/// What has the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Focus {
    pub field: Option<FieldId>,
    pub button: Option<ButtonId>,
}

/// How many characters each field holds, and nothing more.
pub type Typed<'a> = &'a [(FieldId, usize)];

fn cells(text: &str) -> u16 {
    u16::try_from(display_width(text)).unwrap_or(0)
}

/// Draws the prompt over whatever is in `area`.
pub fn draw_prompt(frame: &mut Frame, ctx: &DrawContext<'_>, prompt: &Prompt, placed: &Placed, focus: Focus, typed: Typed<'_>) {
    let rect = placed.frame;
    frame.effects.retain(|effect| !overlaps(effect.area(), rect));
    fill_cells(frame, rect, ctx.full(roles::MENU_BASE));
    draw_border(frame, ctx, rect, prompt);
    for (row, text, tone) in &placed.lines {
        put_text(frame, *row, row.x, row.y, text, tone_style(ctx, *tone));
    }
    frame.cursor = None;
    for (id, label, text, input) in &placed.fields {
        put_text(frame, *label, label.x, label.y, text, ctx.fg(roles::READER_HEADER_LABEL));
        let count = typed.iter().find(|(field, _)| field == id).map_or(0, |(_, count)| *count);
        draw_field(frame, ctx, *input, count, focus.field == Some(*id));
    }
    for (id, rect, label) in &placed.buttons {
        let style = if focus.button == Some(*id) {
            ctx.full(roles::SIDEBAR_SELECTED).add_modifier(Modifier::BOLD)
        } else {
            ctx.full(roles::KEYBAR_CAP)
        };
        fill_cells(frame, *rect, style);
        put_text(frame, *rect, rect.x, rect.y, &format!(" {label} "), style);
    }
}

fn tone_style(ctx: &DrawContext<'_>, tone: Tone) -> Style {
    match tone {
        Tone::Plain => ctx.fg(roles::MENU_BASE),
        Tone::Dim => ctx.fg(roles::LIST_HINT_LABEL),
        Tone::Problem => ctx.fg(roles::FOOTER_ERROR),
    }
}

/// A box of `count` dots in `input`; the last ones when there are more than
/// fit. A focused field is highlighted and carries the caret.
fn draw_field(frame: &mut Frame, ctx: &DrawContext<'_>, input: Rect, count: usize, focused: bool) {
    let style = if focused { ctx.full(roles::SIDEBAR_SELECTED) } else { ctx.full(roles::BASE) };
    fill_cells(frame, input, style);
    // One cell is kept for the caret.
    let room = usize::from(input.width.saturating_sub(1));
    let shown = count.min(room);
    put_text(frame, input, input.x, input.y, &DOT.repeat(shown), style);
    if focused {
        let col = input.x + u16::try_from(shown).unwrap_or(0);
        if col < input.right() {
            frame.cursor = Some(CursorRequest { col, row: input.y, shape: CursorShape::Bar });
        }
    }
}

fn draw_border(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, prompt: &Prompt) {
    let border = ctx.fg(roles::MENU_BORDER);
    let set = ctx.borders;
    let (left, right, top, bottom) = (rect.x, rect.right() - 1, rect.y, rect.bottom() - 1);
    let put = |frame: &mut Frame, col: u16, row: u16, ch: char| put_text(frame, rect, col, row, &ch.to_string(), border);
    for col in left + 1..right {
        put(frame, col, top, set.horizontal);
        put(frame, col, bottom, set.horizontal);
    }
    for row in top + 1..bottom {
        put(frame, left, row, set.vertical);
        put(frame, right, row, set.vertical);
    }
    put(frame, left, top, set.top_left);
    put(frame, right, top, set.top_right);
    put(frame, left, bottom, set.bottom_left);
    put(frame, right, bottom, set.bottom_right);
    put_text(frame, rect, left + 2, top, prompt.title, ctx.fg(roles::SECTION_TITLE));
    let hint_at = right.saturating_sub(cells(prompt.hint) + 1);
    put_text(frame, rect, hint_at.max(left + 2), bottom, prompt.hint, ctx.fg(roles::FOOTER_TEXT));
}

#[cfg(test)]
include!("frame.tests.rs");
