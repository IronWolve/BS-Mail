//! Which rows of a scrolled pane are showing. The drawing and the pointer
//! both ask here, so a click always lands on the row the user sees.

use model::FolderId;

use crate::view_state::{first_row_at_offsets, first_row_showing};
use crate::widgets::FolderRowInfo;

/// The first folder row showing in a body `height` rows tall, keeping the
/// selected folder in view. With no selected row it keeps the scroll as it is.
pub fn folders_first(
    rows: &[FolderRowInfo<'_>],
    offsets: &[u16],
    selected: Option<FolderId>,
    scroll: usize,
    height: u16,
) -> usize {
    match rows.iter().position(|row| Some(row.folder.id) == selected) {
        Some(index) => first_row_at_offsets(offsets, scroll, index, height),
        None => scroll.min(rows.len().saturating_sub(1)),
    }
}

/// The folder row drawn at `y` rows below the top of the body, if any.
pub fn folder_at(offsets: &[u16], first: usize, y: u16) -> Option<usize> {
    let origin = *offsets.get(first)?;
    let wanted = origin.checked_add(y)?;
    offsets.iter().skip(first).position(|offset| *offset == wanted).map(|found| first + found)
}

/// The first message row showing, keeping the selected one in view.
pub fn list_first(scroll: usize, selected: usize, visible: usize, total: usize) -> usize {
    first_row_showing(scroll, selected, visible, total)
}

#[cfg(test)]
include!("viewport.tests.rs");
