//! What a thread adds to the Subject cell: on a thread's row an arrow (open
//! or closed) and the number of messages, on one of its messages an indent.
//! A message on its own adds nothing. The arrow is the cell a click opens or
//! closes the thread by (see `input::thread_keys`).

use super::row_style::RowStyle;
use crate::icons::IconId;
use crate::list_order::RowKind;
use crate::theme::roles;
use crate::widgets::DrawContext;
use crate::widgets::cell_text::Run;

/// Cells the arrow takes at the start of a thread's row, with the blank after
/// it: the cells a click on the arrow can land on.
pub const ARROW_CELLS: u16 = 2;

/// The blank before the subject of a message of an open thread.
const INDENT: &str = "   ";

/// The runs to put before the subject of a row of `kind` in a thread of
/// `count` messages.
pub(super) fn thread_prefix(ctx: &DrawContext<'_>, style: &RowStyle<'_>, kind: RowKind, count: usize) -> Vec<Run> {
    match kind {
        RowKind::Single => Vec::new(),
        RowKind::Child => vec![Run::new(INDENT, style.plain(roles::LIST_ROW))],
        RowKind::Head { open } => {
            let arrow = ctx.icon_inline(if open { IconId::Expanded } else { IconId::Collapsed });
            vec![
                Run::new(format!("{arrow} "), style.accent(roles::LIST_FLAG_UNREAD)),
                Run::new(format!("{count} "), style.plain(roles::LIST_SIZE)),
            ]
        }
    }
}

#[cfg(test)]
include!("thread_cell.tests.rs");
