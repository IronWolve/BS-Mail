//! The folder tree: icon, name and right-aligned unread count per folder,
//! children indented, a bar on the selected folder, and an expand arrow on
//! folders that have children.

use std::collections::HashSet;

use model::{Folder, FolderId, FolderRole};
use surface::{Frame, Rect};

use super::cell_text::{cells, fill_cells, put_cut, put_text};
use super::draw_context::DrawContext;
use crate::format::sidebar_count;
use crate::icons::IconId;
use crate::layout::FolderSpacing;
use crate::theme::roles;

/// Cells a child is indented by, per level.
const INDENT: u16 = 3;
/// Icon slot of a top-level folder, from the left edge of the body.
const ICON_AT: u16 = 1;
/// Name of a top-level folder, from the left edge of the body.
const NAME_AT: u16 = 4;
/// Cells from the end of the name to the expand arrow.
const ARROW_GAP: u16 = 2;

/// One folder that is currently visible.
#[derive(Debug, Clone, Copy)]
pub struct FolderRowInfo<'a> {
    pub folder: &'a Folder,
    /// 0 for a top-level folder.
    pub depth: u16,
    pub has_children: bool,
}

/// What to draw: the visible rows, which is selected, and from which row.
pub struct FolderTreeView<'a> {
    pub rows: &'a [FolderRowInfo<'a>],
    pub selected: Option<FolderId>,
    /// The role of the selection bar: `sidebar.selected`, or its dimmer
    /// `sidebar.selected.unfocused` while the keys are not in the folders.
    pub bar: &'static str,
    pub spacing: FolderSpacing,
    /// Index of the first row shown.
    pub first: usize,
}

/// The folders to show, in list order: a folder is hidden when any ancestor
/// is collapsed. A folder whose parent is not in the list counts as top level.
pub fn visible_rows(folders: &[Folder]) -> Vec<FolderRowInfo<'_>> {
    let parents: HashSet<FolderId> = folders.iter().filter_map(|f| f.parent).collect();
    folders
        .iter()
        .filter_map(|folder| {
            let (depth, hidden) = ancestry(folders, folder);
            (!hidden).then_some(FolderRowInfo {
                folder,
                depth,
                has_children: parents.contains(&folder.id),
            })
        })
        .collect()
}

/// Depth of `folder` and whether a collapsed ancestor hides it.
fn ancestry(folders: &[Folder], folder: &Folder) -> (u16, bool) {
    let mut depth = 0u16;
    let mut hidden = false;
    let mut next = folder.parent;
    for _ in 0..folders.len() {
        let Some(parent) = next.and_then(|id| folders.iter().find(|f| f.id == id)) else {
            break;
        };
        hidden |= !parent.expanded;
        depth += 1;
        next = parent.parent;
    }
    (depth, hidden)
}

/// The row offset of each folder from the top of the body.
pub fn row_offsets(rows: &[FolderRowInfo<'_>], spacing: FolderSpacing) -> Vec<u16> {
    let mut y = 0u16;
    let mut offsets = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        if index > 0 && spacing == FolderSpacing::Roomy && row.depth == 0 {
            y += 1;
        }
        offsets.push(y);
        y += 1;
    }
    offsets
}

/// Rows the whole tree needs.
pub fn content_rows(rows: &[FolderRowInfo<'_>], spacing: FolderSpacing) -> u16 {
    row_offsets(rows, spacing).last().map_or(0, |last| last + 1)
}

fn icon_for(row: &FolderRowInfo<'_>) -> (IconId, &'static str) {
    match row.folder.role {
        FolderRole::Inbox => (IconId::Inbox, roles::ICON_INBOX),
        FolderRole::Starred => (IconId::Star, roles::ICON_STARRED),
        FolderRole::Sent => (IconId::Sent, roles::ICON_SENT),
        FolderRole::Drafts => (IconId::Draft, roles::ICON_DRAFTS),
        FolderRole::Archive => (IconId::Archive, roles::ICON_ARCHIVE),
        FolderRole::Alerts => (IconId::Bell, roles::ICON_ALERTS),
        FolderRole::Spam => (IconId::Ban, roles::ICON_SPAM),
        FolderRole::Trash => (IconId::Trash, roles::ICON_TRASH),
        FolderRole::Plain if row.depth > 0 => (IconId::FolderOutline, roles::ICON_FOLDER),
        FolderRole::Plain if row.has_children && row.folder.expanded => {
            (IconId::FolderOpen, roles::ICON_FOLDER)
        }
        FolderRole::Plain => (IconId::Folder, roles::ICON_FOLDER),
    }
}

/// Draws the visible part of the tree into `body`, one folder per row.
pub fn draw_folder_tree(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    body: Rect,
    view: &FolderTreeView<'_>,
) {
    let offsets = row_offsets(view.rows, view.spacing);
    let Some(base) = offsets.get(view.first).copied() else {
        return;
    };
    for (row, offset) in view.rows.iter().zip(&offsets).skip(view.first) {
        let dy = offset - base;
        if dy >= body.height {
            break;
        }
        let selected = view.selected == Some(row.folder.id);
        draw_row(frame, ctx, Rect::new(body.x, body.y + dy, body.width, 1), row, selected.then_some(view.bar));
    }
}

/// The unread count and where it starts. A name always keeps at least the
/// one cell its ellipsis needs: when the count would take that, the count
/// is left out instead.
fn count_slot(
    ctx: &DrawContext<'_>,
    row: &FolderRowInfo<'_>,
    line: Rect,
    name_x: u16,
) -> (Option<String>, u16) {
    let count = sidebar_count(&ctx.config.appearance, row.folder.unread);
    let count_x = count
        .as_ref()
        .map_or(line.right(), |text| line.right().saturating_sub(cells(text)));
    if count_x.saturating_sub(1).saturating_sub(name_x) == 0 {
        return (None, line.right());
    }
    (count, count_x)
}

fn draw_row(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    line: Rect,
    row: &FolderRowInfo<'_>,
    bar: Option<&str>,
) {
    if let Some(bar) = bar {
        fill_cells(frame, line, ctx.full(bar));
    }
    let pick = |role: &str| match bar {
        Some(bar) => ctx.fg(bar),
        None => ctx.fg(role),
    };
    let shift = INDENT.saturating_mul(row.depth);
    let (icon, icon_role) = icon_for(row);
    let icon_x = line.x.saturating_add(ICON_AT).saturating_add(shift);
    put_text(frame, line, icon_x, line.y, ctx.icon(icon), pick(icon_role));
    let name_x = line.x.saturating_add(NAME_AT).saturating_add(shift);
    let (count, count_x) = count_slot(ctx, row, line, name_x);
    let room = count_x.saturating_sub(1).saturating_sub(name_x);
    let name_end = put_cut(frame, line, name_x, line.y, &row.folder.name, room, pick(roles::SIDEBAR_ITEM));
    if row.has_children {
        let arrow = if row.folder.expanded { IconId::Expanded } else { IconId::Collapsed };
        let text = ctx.icon_inline(arrow);
        let arrow_x = name_end.saturating_add(ARROW_GAP);
        if arrow_x.saturating_add(cells(text)) < count_x {
            put_text(frame, line, arrow_x, line.y, text, pick(roles::SIDEBAR_CHEVRON));
        }
    }
    if let Some(text) = count {
        put_text(frame, line, count_x, line.y, &text, pick(roles::SIDEBAR_COUNT));
    }
}

#[cfg(test)]
include!("folder_tree.tests.rs");
