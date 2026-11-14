//! What is under the pointer: a folder row, a message row, the reader.

use surface::Rect;

use super::viewport::folder_at;
use crate::widgets::button_at;
use crate::layout::{Layout, SectionKind};

/// The part of the screen a pointer position falls on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    /// A folder row, as an index into the visible folder rows.
    Folder(usize),
    /// Somewhere in the sidebar that is not a folder row.
    Sidebar,
    /// A message row, as an index into the page's envelopes.
    Message(usize),
    /// The list's title or column header.
    ListChrome,
    Reader,
    /// A key bar button, by the name of its action.
    KeyBar(&'static str),
    Nothing,
}

/// What the scrolled panes show, so rows can be matched to positions.
pub struct HitView<'a> {
    pub layout: &'a Layout,
    pub folder_offsets: &'a [u16],
    pub folders_first: usize,
    pub list_first: usize,
    pub messages: usize,
    /// The actions the key bar offers, in order.
    pub key_bar_actions: &'a [String],
}

/// What lies at column `col`, row `row` of the grid.
pub fn hit_at(view: &HitView<'_>, col: u16, row: u16) -> Hit {
    if let Some(bar) = view.layout.key_bar
        && inside(bar, col, row)
    {
        return button_at(bar, view.key_bar_actions, col).map_or(Hit::Nothing, Hit::KeyBar);
    }
    if let Some(sidebar) = &view.layout.sidebar
        && inside(sidebar.rect, col, row)
    {
        let folders = sidebar.section(SectionKind::Folders);
        return match folders {
            Some(section) if inside(section.body, col, row) => {
                folder_at(view.folder_offsets, view.folders_first, row - section.body.y)
                    .map_or(Hit::Sidebar, Hit::Folder)
            }
            _ => Hit::Sidebar,
        };
    }
    if let Some(list) = &view.layout.list
        && inside(list.rect, col, row)
    {
        if inside(list.rows, col, row) {
            let index = view.list_first + usize::from(row - list.rows.y);
            return if index < view.messages { Hit::Message(index) } else { Hit::Nothing };
        }
        return Hit::ListChrome;
    }
    match &view.layout.reader {
        Some(reader) if inside(reader.rect, col, row) => Hit::Reader,
        _ => Hit::Nothing,
    }
}

fn inside(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x && row >= rect.y && col < rect.right() && row < rect.bottom()
}

#[cfg(test)]
include!("hit.tests.rs");
