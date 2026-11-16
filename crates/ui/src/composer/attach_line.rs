//! The Attach row of the composer: the files attached as chips (name and
//! size), what is to come from a forwarded original, and two keycap buttons at
//! the right end, Add (`Ctrl+O`) and, while a file is chosen on the row,
//! Remove (`Delete`). `attach_rects` says where each part sits, so the drawing
//! and the pointer both ask it and a click lands on what is seen.

use surface::{Frame, Modifier, Rect};

use super::buttons::draw_one;
use super::draw::LABEL_CELLS;
use super::form::{ComposeForm, Field};
use crate::format::{ELLIPSIS_CHAR, truncate};
use crate::theme::roles;
use crate::widgets::{DrawContext, fill_cells, put_text};

/// The key and word of the button that opens the path box.
pub const ADD: (&str, &str) = ("Ctrl+O", "Add");
/// The key and word of the button that takes the chosen file off.
pub const REMOVE: (&str, &str) = ("Del", "Remove");

/// The most cells of a chip's text.
const CHIP_CELLS: usize = 30;
/// Cells between two chips.
const GAP: u16 = 2;
/// Cells kept after a chip for the words that say how many more there are.
const MORE_CELLS: u16 = 10;

/// Where the parts of the Attach row sit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachRects {
    /// Each chip that fits and the index of its file.
    pub chips: Vec<(Rect, usize)>,
    /// The words that say how many files did not fit, when some did not.
    pub more: Option<(Rect, usize)>,
    pub add: Rect,
    /// Only while a file is chosen on the row.
    pub remove: Option<Rect>,
}

fn cells(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

/// A size in the units of a mail program: `512 B`, `3.4 KB`, `2.1 MB`.
pub fn human_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// The text of one chip: the name, cut to fit, and the size.
pub fn chip_text(form: &ComposeForm, at: usize) -> String {
    let Some(file) = form.files().get(at) else {
        return String::new();
    };
    let size = format!(" ({})", human_size(file.data.len()));
    let name = truncate(&file.name, CHIP_CELLS.saturating_sub(size.chars().count()), ELLIPSIS_CHAR);
    format!("{name}{size}")
}

fn button_width((key, word): (&str, &str)) -> u16 {
    cells(key) + 2 + 1 + cells(word)
}

/// The parts of the Attach row `row` (the whole row, label included) of `form`.
pub fn attach_rects(row: Rect, form: &ComposeForm) -> AttachRects {
    let right = row.right();
    let add = Rect::new(right.saturating_sub(button_width(ADD)), row.y, button_width(ADD), 1);
    let chosen = form.focus == Field::Attach && form.attach_cursor.is_some();
    let remove = chosen.then(|| {
        let width = button_width(REMOVE);
        Rect::new(add.x.saturating_sub(width + GAP), row.y, width, 1)
    });
    let start = row.x + LABEL_CELLS;
    let end = remove.map_or(add.x, |rect| rect.x).saturating_sub(GAP);
    let mut chips = Vec::new();
    let mut x = start;
    let mut hidden = 0;
    let total = form.files().len();
    for at in 0..total {
        let width = cells(&chip_text(form, at));
        // Room is kept for the `+N more` that would follow, except after the last chip.
        let needed = width + if at + 1 < total { MORE_CELLS } else { 0 };
        let fits = x + needed <= end || (chips.is_empty() && x + width <= end);
        if !fits {
            hidden = total - at;
            break;
        }
        chips.push((Rect::new(x, row.y, width, 1), at));
        x += width + GAP;
    }
    let more = (hidden > 0).then(|| (Rect::new(x, row.y, cells(&more_text(hidden)), 1), hidden));
    AttachRects { chips, more, add, remove }
}

fn more_text(hidden: usize) -> String {
    format!("+{hidden} more")
}

/// Which chip is at the cell, as the index of its file.
pub fn chip_at(rects: &AttachRects, col: u16, row: u16) -> Option<usize> {
    rects.chips.iter().find(|(rect, _)| row == rect.y && col >= rect.x && col < rect.right()).map(|(_, at)| *at)
}

pub fn inside(rect: Rect, col: u16, row: u16) -> bool {
    row == rect.y && col >= rect.x && col < rect.right()
}

/// Draws the row: the label, the chips, what is left out, what hint there is,
/// and the buttons.
pub fn draw_attach(frame: &mut Frame, ctx: &DrawContext<'_>, row: Rect, form: &ComposeForm) {
    let focused = form.focus == Field::Attach;
    let label_style = if focused { ctx.fg(roles::SECTION_TITLE) } else { ctx.fg(roles::READER_HEADER_LABEL) };
    put_text(frame, row, row.x, row.y, Field::Attach.label(), label_style);
    let rects = attach_rects(row, form);
    let value = ctx.fg(roles::READER_HEADER_VALUE);
    for (rect, at) in &rects.chips {
        let chosen = focused && form.attach_cursor == Some(*at);
        let style = if chosen { value.add_modifier(Modifier::REVERSED) } else { value };
        fill_cells(frame, *rect, if chosen { ctx.full(roles::SIDEBAR_SELECTED) } else { ctx.full(roles::BASE) });
        put_text(frame, *rect, rect.x, rect.y, &chip_text(form, *at), style);
    }
    if let Some((rect, hidden)) = &rects.more {
        put_text(frame, row, rect.x, rect.y, &more_text(*hidden), ctx.fg(roles::LIST_HINT_LABEL));
    }
    if form.files().is_empty() {
        let hint = if form.can_focus_attach() { "" } else { "no files attached" };
        put_text(frame, row, row.x + LABEL_CELLS, row.y, hint, ctx.fg(roles::LIST_HINT_LABEL));
    }
    draw_one(frame, ctx, row, rects.add, ADD, true, form.path.is_some());
    if let Some(remove) = rects.remove {
        draw_one(frame, ctx, row, remove, REMOVE, true, false);
    }
}

#[cfg(test)]
include!("attach_line.tests.rs");
