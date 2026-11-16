//! The flag cell: three cells wide, with the answered or forwarded mark on
//! the left, the alert or unread dot in the middle, where it has always been,
//! and the attachment mark on the right. The marks take no room from any other
//! column, and a cell narrower than three (a list too narrow for its columns)
//! shows the middle one only.

use super::cells::RowInfo;
use super::row_style::RowStyle;
use crate::icons::IconId;
use crate::layout::ColumnSlot;
use crate::theme::roles;
use crate::widgets::cell_text::{Align, put_aligned};
use crate::widgets::DrawContext;

/// Cells the flag cell needs to show all three places.
const FULL_WIDTH: u16 = 3;

pub(super) fn draw_flag_cell(
    frame: &mut surface::Frame,
    ctx: &DrawContext<'_>,
    style: &RowStyle<'_>,
    row: &RowInfo<'_>,
    slot: &ColumnSlot,
) {
    let flags = row.envelope.flags;
    let middle = (slot.x, row.y, slot.width);
    if flags.alert {
        let icon = ctx.icon_inline(IconId::Alert);
        put_aligned(frame, row.clip, middle, icon, Align::Center, style.accent(roles::LIST_FLAG_ALERT));
    } else if !flags.seen {
        let icon = ctx.icon_inline(IconId::Dot);
        put_aligned(frame, row.clip, middle, icon, Align::Center, style.accent(roles::LIST_FLAG_UNREAD));
    }
    if slot.width < FULL_WIDTH {
        return;
    }
    let mark = if flags.answered {
        Some(IconId::Reply)
    } else {
        flags.forwarded.then_some(IconId::Forward)
    };
    if let Some(mark) = mark {
        let at = (slot.x, row.y, 1);
        let icon = ctx.icon_inline(mark);
        put_aligned(frame, row.clip, at, icon, Align::Left, style.accent(roles::LIST_FLAG_ANSWERED));
    }
    if row.envelope.has_attachments {
        let at = (slot.x + slot.width - 1, row.y, 1);
        let icon = ctx.icon_inline(IconId::Attachment);
        put_aligned(frame, row.clip, at, icon, Align::Left, style.accent(roles::LIST_ATTACHMENT));
    }
}

#[cfg(test)]
include!("flag_cell.tests.rs");
