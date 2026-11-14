//! The help screen: a box over the middle of the screen listing the keys
//! and the mouse actions that work now.
//!
//! The list is data, in one place, so it is easy to keep honest: a row here
//! is a promise that the key does that.

use surface::{Frame, Rect};

use super::cell_text::{fill_cells, put_text};
use super::draw_context::DrawContext;
use crate::theme::roles;

/// What each key or gesture does, one row each: the keys, then the meaning.
pub const HELP_ROWS: &[(&str, &str)] = &[
    ("Up Down  j k", "move in the pane that has focus"),
    ("PgUp PgDn  Space", "a page at a time (Space in the reader)"),
    ("Home End", "first and last"),
    ("Tab  Shift+Tab", "change pane"),
    ("Enter  Esc", "go in a pane, come back out"),
    ("Right  Left", "open and close a folder; in the other panes, in and out"),
    ("click", "select a folder or message"),
    ("wheel", "scroll the pane under the pointer"),
    ("Space (list)", "mark a message; u * d then act on all marked, Esc clears"),
    ("u", "mark read or unread"),
    ("*", "star or unstar"),
    ("d", "delete"),
    ("m", "move to another folder"),
    ("s", "sync now"),
    ("c  F2", "write a new message"),
    ("r  R  f", "reply, reply to everyone, forward"),
    ("Ctrl+Enter  F10", "send the message being written"),
    ("drag a bar", "resize the sidebar and the list and reader split"),
    ("Ctrl + - 0  Ctrl wheel", "text size, and back to normal"),
    ("F11", "full screen"),
    ("?", "this help"),
    ("q  Ctrl+Q", "quit"),
];

const TITLE: &str = " Keys ";
const CLOSE_HINT: &str = " any key closes ";
/// Blank cells between the box edge and the text, and between the columns.
const PAD: u16 = 2;
const GAP: u16 = 3;

/// The box for a grid: centered, as large as the list wants, never larger
/// than the grid leaves room for.
pub fn help_rect(grid: Rect) -> Rect {
    let keys = HELP_ROWS.iter().map(|(keys, _)| cells(keys)).max().unwrap_or(0);
    let meaning = HELP_ROWS.iter().map(|(_, text)| cells(text)).max().unwrap_or(0);
    let wanted_width = keys + GAP + meaning + PAD * 2 + 2;
    let wanted_height = u16::try_from(HELP_ROWS.len()).unwrap_or(u16::MAX) + 4;
    let width = wanted_width.min(grid.width);
    let height = wanted_height.min(grid.height);
    Rect::new(
        grid.x + (grid.width - width) / 2,
        grid.y + (grid.height - height) / 2,
        width,
        height,
    )
}

fn cells(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

/// Draws the help box over `grid`. Effects under the box are dropped, so a
/// big title or a keycap does not show through it.
pub fn draw_help(frame: &mut Frame, ctx: &DrawContext<'_>, grid: Rect) {
    let rect = help_rect(grid);
    if rect.width < 8 || rect.height < 4 {
        return;
    }
    frame.effects.retain(|effect| !overlaps(effect.area(), rect));
    fill_cells(frame, rect, ctx.full(roles::BASE));
    draw_frame(frame, ctx, rect);
    let keys_width = HELP_ROWS.iter().map(|(keys, _)| cells(keys)).max().unwrap_or(0);
    let text_clip = Rect::new(rect.x + 1, rect.y + 1, rect.width - 2, rect.height - 2);
    for (index, (keys, meaning)) in HELP_ROWS.iter().enumerate() {
        let row = rect.y + 2 + u16::try_from(index).unwrap_or(u16::MAX);
        if row >= rect.bottom() - 1 {
            break;
        }
        let left = rect.x + 1 + PAD;
        put_text(frame, text_clip, left, row, keys, ctx.fg(roles::LIST_HINT_KEY));
        put_text(frame, text_clip, left + keys_width + GAP, row, meaning, ctx.fg(roles::BODY_TEXT));
    }
}

fn draw_frame(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect) {
    let border = ctx.fg(roles::BORDER_FOCUSED);
    let set = ctx.borders;
    let (left, right) = (rect.x, rect.right() - 1);
    let (top, bottom) = (rect.y, rect.bottom() - 1);
    let clip = rect;
    for col in left + 1..right {
        put_text(frame, clip, col, top, &set.horizontal.to_string(), border);
        put_text(frame, clip, col, bottom, &set.horizontal.to_string(), border);
    }
    for row in top + 1..bottom {
        put_text(frame, clip, left, row, &set.vertical.to_string(), border);
        put_text(frame, clip, right, row, &set.vertical.to_string(), border);
    }
    put_text(frame, clip, left, top, &set.top_left.to_string(), border);
    put_text(frame, clip, right, top, &set.top_right.to_string(), border);
    put_text(frame, clip, left, bottom, &set.bottom_left.to_string(), border);
    put_text(frame, clip, right, bottom, &set.bottom_right.to_string(), border);
    put_text(frame, clip, left + 2, top, TITLE, ctx.fg(roles::SECTION_TITLE));
    let hint_at = right.saturating_sub(cells(CLOSE_HINT) + 2);
    put_text(frame, clip, hint_at, bottom, CLOSE_HINT, ctx.fg(roles::FOOTER_TEXT));
}

pub(crate) fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.right() && b.x < a.right() && a.y < b.bottom() && b.y < a.bottom()
}

#[cfg(test)]
include!("help_overlay.tests.rs");
