//! What each column of one row shows.

use model::{Envelope, MailDate};
use surface::{Frame, Rect};

use super::row_style::RowStyle;
use crate::format::{DateFormat, format_address, format_list_date, format_size, is_recent};
use crate::icons::IconId;
use crate::layout::{ColumnId, ColumnSlot};
use crate::theme::roles;
use crate::widgets::cell_text::{put_aligned, put_cut, put_runs_cut};
use crate::widgets::list_header::column_align;
use crate::widgets::subject_style::subject_runs;
use crate::widgets::DrawContext;

/// One row to draw: its envelope, its number and where it goes.
pub struct RowInfo<'a> {
    pub envelope: &'a Envelope,
    /// Position in the folder, 1-based.
    pub number: u64,
    pub y: u16,
    /// The list's text width; nothing is written outside it.
    pub clip: Rect,
    pub short_date: bool,
    pub now: MailDate,
}

/// Draws the cell of `slot` for `row`.
pub fn draw_cell(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    style: &RowStyle<'_>,
    row: &RowInfo<'_>,
    slot: &ColumnSlot,
) {
    let at = (slot.x, row.y, slot.width);
    let align = column_align(slot.id);
    let flags = row.envelope.flags;
    match slot.id {
        ColumnId::Num => {
            let text = row.number.to_string();
            put_aligned(frame, row.clip, at, &text, align, style.plain(roles::LIST_ROW));
        }
        ColumnId::Flag if flags.alert => {
            let icon = ctx.icon_inline(IconId::Alert);
            put_aligned(frame, row.clip, at, icon, align, style.accent(roles::LIST_FLAG_ALERT));
        }
        ColumnId::Flag if !flags.seen => {
            let icon = ctx.icon_inline(IconId::Dot);
            put_aligned(frame, row.clip, at, icon, align, style.accent(roles::LIST_FLAG_UNREAD));
        }
        ColumnId::Star if flags.flagged => {
            let icon = ctx.icon_inline(IconId::Star);
            put_aligned(frame, row.clip, at, icon, align, style.accent(roles::LIST_STAR));
        }
        ColumnId::Flag | ColumnId::Star => {}
        ColumnId::From => draw_from(frame, ctx, style, row, slot),
        ColumnId::Subject => draw_subject(frame, ctx, style, row, slot),
        ColumnId::Date => draw_date(frame, ctx, style, row, slot),
        ColumnId::Size => {
            let text = format_size(row.envelope.size, ctx.config.list.size_format);
            put_aligned(frame, row.clip, at, &text, align, style.plain(roles::LIST_SIZE));
        }
    }
}

fn draw_from(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    style: &RowStyle<'_>,
    row: &RowInfo<'_>,
    slot: &ColumnSlot,
) {
    let Some(from) = row.envelope.from.first() else {
        return;
    };
    let text = format_address(from, ctx.config.list.from_format);
    let plain = style.plain(roles::LIST_ROW);
    put_cut(frame, row.clip, slot.x, row.y, &text, slot.width, plain);
}

fn draw_subject(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    style: &RowStyle<'_>,
    row: &RowInfo<'_>,
    slot: &ColumnSlot,
) {
    let base = style.plain(roles::LIST_ROW);
    let runs = subject_runs(ctx, &row.envelope.subject, base, &|role| style.accent(role), true);
    put_runs_cut(frame, row.clip, slot.x, row.y, &runs, slot.width);
}

fn draw_date(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    style: &RowStyle<'_>,
    row: &RowInfo<'_>,
    slot: &ColumnSlot,
) {
    let format = DateFormat::from_config(&ctx.config.list);
    let text = format_list_date(row.envelope.date, row.now, &format, row.short_date);
    let role = if is_recent(row.envelope.date, row.now) {
        roles::LIST_DATE_RECENT
    } else {
        roles::LIST_DATE
    };
    put_cut(frame, row.clip, slot.x, row.y, &text, slot.width, style.plain(role));
}

#[cfg(test)]
include!("cells.tests.rs");
