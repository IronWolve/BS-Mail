//! The message list's column header: each label in its column's slot with
//! the column's alignment, and a thin bar in the gutter after the columns
//! that end a group.

use surface::{Frame, Rect};

use super::cell_text::{Align, put_aligned, put_text};
use super::draw_context::DrawContext;
use crate::layout::{ColumnId, ColumnLayout};
use crate::theme::roles;

/// The label a column shows in the header.
pub fn column_label(id: ColumnId) -> &'static str {
    match id {
        ColumnId::Num => "#",
        ColumnId::Flag => "F",
        ColumnId::Star => "S",
        ColumnId::From => "From",
        ColumnId::Subject => "Subject",
        ColumnId::Date => "Date",
        ColumnId::Size => "Size",
    }
}

/// How a column places its header label and its cell text: numbers and sizes
/// on the right, flags and stars centered, the rest on the left.
pub fn column_align(id: ColumnId) -> Align {
    match id {
        ColumnId::Num | ColumnId::Size => Align::Right,
        ColumnId::Flag | ColumnId::Star => Align::Center,
        ColumnId::From | ColumnId::Subject | ColumnId::Date => Align::Left,
    }
}

/// Columns whose right-hand gutter holds a bar. The Flag and Star columns
/// are one group, so there is no bar between them.
fn bar_after(id: ColumnId) -> bool {
    matches!(
        id,
        ColumnId::Num | ColumnId::Star | ColumnId::From | ColumnId::Subject | ColumnId::Date
    )
}

/// Draws the header into the one-row `rect` (the list's text width).
/// `from_label` is what the sender column is called over these rows: `From`,
/// or `To` over a list of sent mail.
pub fn draw_list_header(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    rect: Rect,
    columns: &ColumnLayout,
    from_label: &str,
) {
    if rect.is_empty() {
        return;
    }
    let label_style = ctx.fg(roles::LIST_HEADER);
    let bar = ctx.borders.vertical.to_string();
    let bar_style = ctx.fg(roles::LIST_HEADER_BAR);
    for (index, slot) in columns.slots.iter().enumerate() {
        let at = (slot.x, rect.y, slot.width);
        let label = if slot.id == ColumnId::From { from_label } else { column_label(slot.id) };
        put_aligned(frame, rect, at, label, column_align(slot.id), label_style);
        let has_next = index + 1 < columns.slots.len();
        if bar_after(slot.id) && has_next {
            put_text(frame, rect, slot.x.saturating_add(slot.width), rect.y, &bar, bar_style);
        }
    }
}

#[cfg(test)]
include!("list_header.tests.rs");
