//! The visible rows of the message list.

use model::{Envelope, MailDate};
use surface::{Frame, Rect};

use super::cells::{RowInfo, draw_cell};
use super::row_style::RowStyle;
use crate::layout::ColumnLayout;
use crate::theme::roles;
use crate::widgets::DrawContext;
use crate::widgets::cell_text::fill_cells;

/// What to draw: the page of envelopes, the columns and the position.
pub struct ListRowsView<'a> {
    pub envelopes: &'a [Envelope],
    pub columns: &'a ColumnLayout,
    /// Index in `envelopes` of the selected row.
    pub selected: Option<usize>,
    /// Position in the folder of the page's first envelope, 0-based.
    pub offset: u32,
    /// Index in `envelopes` of the first row shown.
    pub first: usize,
    pub now: MailDate,
}

/// Draws one envelope per row of `rows`, top down, as many as fit. Rows past
/// the last envelope are left as they are.
pub fn draw_list_rows(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    rows: Rect,
    view: &ListRowsView<'_>,
) {
    let shown = view.envelopes.iter().enumerate().skip(view.first);
    for (line, (index, envelope)) in shown.take(usize::from(rows.height)).enumerate() {
        let y = rows.y.saturating_add(u16::try_from(line).unwrap_or(u16::MAX));
        let selected = view.selected == Some(index);
        let style = RowStyle::new(ctx, selected);
        if selected {
            let bar = Rect::new(rows.x, y, rows.width, 1);
            fill_cells(frame, bar, ctx.full(roles::LIST_SELECTED));
        }
        let info = RowInfo {
            envelope,
            number: u64::from(view.offset) + index as u64 + 1,
            y,
            clip: rows,
            short_date: view.columns.short_date,
            now: view.now,
        };
        for slot in &view.columns.slots {
            draw_cell(frame, ctx, &style, &info, slot);
        }
    }
}

#[cfg(test)]
include!("draw.tests.rs");
