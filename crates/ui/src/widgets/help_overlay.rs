//! The help screen: a box over the middle of the screen listing the keys
//! and the mouse actions that work now, in the host that shows it. On a grid
//! too short for every row the list scrolls and the box says so. The rows
//! themselves are in `help_rows`.

use surface::{Capabilities, Frame, Rect};

use super::cell_text::{fill_cells, put_text};
use super::draw_context::DrawContext;
use super::help_rows::help_rows;
use crate::keys::KeyMap;
use crate::theme::roles;

const TITLE: &str = " Keys ";
const CLOSE_HINT: &str = " any key closes ";
/// The same, with where keys are changed, when the box is wide enough.
const REBIND_HINT: &str = " any key closes   change keys in config.toml (the keys table); --print-config lists them ";
/// The hint when some rows are out of sight and the arrow keys reach them.
const SCROLL_HINT: &str = " Up Down scroll, other key closes ";
/// Blank cells between the box edge and the text, and between the columns.
const PAD: u16 = 2;
const GAP: u16 = 3;

type Row = (String, String);

/// The box for a grid: centered, as large as the rows want, never larger
/// than the grid leaves room for.
pub fn help_rect(grid: Rect, rows: &[Row]) -> Rect {
    let keys = rows.iter().map(|(keys, _)| cells(keys)).max().unwrap_or(0);
    let meaning = rows.iter().map(|(_, text)| cells(text)).max().unwrap_or(0);
    let wanted_width = keys + GAP + meaning + PAD * 2 + 2;
    let wanted_height = u16::try_from(rows.len()).unwrap_or(u16::MAX).saturating_add(4);
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

/// How many rows the box shows at once: its height less the two border rows
/// and the blank row under the top border.
pub fn help_capacity(rect: Rect) -> usize {
    usize::from(rect.height.saturating_sub(3))
}

/// The most the help list can scroll on `grid` for a host with
/// `capabilities`: how many rows are out of sight when it is at the top.
pub fn help_max_scroll(grid: Rect, capabilities: &Capabilities, keys: &KeyMap) -> usize {
    let rows = help_rows(capabilities, keys);
    let rect = help_rect(grid, &rows);
    rows.len().saturating_sub(help_capacity(rect))
}

/// Draws the help box over `grid`, the list starting at row `scroll` when
/// the grid is too short for all of it. Effects under the box are dropped,
/// so a big title or a keycap does not show through it.
pub fn draw_help(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    grid: Rect,
    capabilities: &Capabilities,
    scroll: usize,
) {
    let rows = help_rows(capabilities, ctx.keys);
    let rect = help_rect(grid, &rows);
    if rect.width < 8 || rect.height < 4 {
        return;
    }
    frame.effects.retain(|effect| !overlaps(effect.area(), rect));
    fill_cells(frame, rect, ctx.full(roles::BASE));
    let capacity = help_capacity(rect);
    let first = scroll.min(rows.len().saturating_sub(capacity));
    let shown = rows.len().min(first + capacity);
    let clipped = rows.len() > capacity;
    draw_frame(frame, ctx, rect, clipped.then_some((first, shown, rows.len())));
    let keys_width = rows.iter().map(|(keys, _)| cells(keys)).max().unwrap_or(0);
    let text_clip = Rect::new(rect.x + 1, rect.y + 1, rect.width - 2, rect.height - 2);
    for (index, (keys, meaning)) in rows[first..shown].iter().enumerate() {
        let row = rect.y + 2 + u16::try_from(index).unwrap_or(u16::MAX);
        let left = rect.x + 1 + PAD;
        put_text(frame, text_clip, left, row, keys, ctx.fg(roles::LIST_HINT_KEY));
        put_text(frame, text_clip, left + keys_width + GAP, row, meaning, ctx.fg(roles::BODY_TEXT));
    }
}

/// The frame, its title and its hint. `range` is the rows shown (first, end,
/// total) when the list is cut: the title then names them and the hint says
/// the arrow keys scroll.
fn draw_frame(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, range: Option<(usize, usize, usize)>) {
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
    let title = match range {
        Some((first, end, total)) => format!(" Keys {}-{end} of {total} ", first + 1),
        None => TITLE.to_string(),
    };
    put_text(frame, clip, left + 2, top, &title, ctx.fg(roles::SECTION_TITLE));
    let room = rect.width.saturating_sub(cells(&title) + 8);
    let hint = match range {
        Some(_) => SCROLL_HINT,
        None if cells(REBIND_HINT) <= room => REBIND_HINT,
        None => CLOSE_HINT,
    };
    let hint_at = right.saturating_sub(cells(hint) + 2);
    put_text(frame, clip, hint_at, bottom, hint, ctx.fg(roles::FOOTER_TEXT));
}

pub(crate) fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.right() && b.x < a.right() && a.y < b.bottom() && b.y < a.bottom()
}

#[cfg(test)]
include!("help_overlay.tests.rs");
