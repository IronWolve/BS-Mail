//! The scrollbars of the list, the reader and the folder tree as the pointer
//! meets them: where each one is, what it shows, and what a press on its
//! thumb or its track does. The bars are drawn where the content overflows,
//! in the cell beside the pane's text, so no column or line of text gives up
//! room for one.

use surface::Rect;

use super::motion::folders_height;
use super::viewport::{folders_first_at, last_folders_first};
use crate::app_view::AppView;
use crate::layout::{Layout, ListRegion, ReaderRegion, SidebarLayout, SectionKind};
use crate::widgets::{first_for_top, row_offsets, thumb_span, visible_rows};

/// The panes that have a scrollbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarTarget {
    Folders,
    List,
    Reader,
}

/// One scrollbar: its column and rows, and what it shows, all in rows of its
/// own pane's content (the folder tree counts the blank rows between folders).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bar {
    pub target: BarTarget,
    pub col: u16,
    pub rows: Rect,
    pub first: usize,
    pub visible: usize,
    pub total: usize,
}

/// A thumb being dragged: which bar, and how far below the thumb's top the
/// pointer took hold of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BarDrag {
    pub target: BarTarget,
    pub grab: u16,
}

impl Bar {
    fn new(target: BarTarget, col: u16, rows: Rect, (first, total): (usize, usize)) -> Option<Self> {
        let visible = usize::from(rows.height);
        (total > visible && visible > 0).then_some(Self { target, col, rows, first, visible, total })
    }

    pub(crate) fn holds(&self, col: u16, row: u16) -> bool {
        col == self.col && row >= self.rows.y && row < self.rows.bottom()
    }
}

impl AppView {
    /// The list's bar, in the cell right of its text.
    pub(crate) fn list_bar(&self, list: &ListRegion) -> Option<Bar> {
        if list.rect.width < 3 {
            return None;
        }
        let total = self.shown_page().map_or(0, |page| page.envelopes.len());
        let first = self.list_view_first(usize::from(list.rows.height), total);
        Bar::new(BarTarget::List, list.text.right(), list.rows, (first, total))
    }

    /// The folder tree's bar, in the cell right of the sidebar's text.
    pub(crate) fn folders_bar(&self, sidebar: &SidebarLayout) -> Option<Bar> {
        let body = sidebar.section(SectionKind::Folders)?.body;
        let rows = visible_rows(&self.mail.folders);
        let offsets = row_offsets(&rows, sidebar.spacing);
        let first = self.folders_view_first(&rows, &offsets, body.height);
        let total = offsets.last().map_or(0, |last| usize::from(*last) + 1);
        let at = offsets.get(first).map_or(0, |offset| usize::from(*offset));
        Bar::new(BarTarget::Folders, body.right(), body, (at, total))
    }

    /// The reader's bar, in the body's last column.
    fn reader_bar(&self, reader: &ReaderRegion, layout: &Layout) -> Option<Bar> {
        let (total, _) = self.reader_extent(layout)?;
        let first = self.scroll.reader.min(total.saturating_sub(usize::from(reader.body.height)));
        Bar::new(BarTarget::Reader, reader.body.right().saturating_sub(1), reader.body, (first, total))
    }

    /// Every bar that is showing.
    pub(crate) fn scrollbars(&self, layout: &Layout) -> Vec<Bar> {
        let list = layout.list.as_ref().and_then(|list| self.list_bar(list));
        let folders = layout.sidebar.as_ref().and_then(|sidebar| self.folders_bar(sidebar));
        let reader = layout.reader.as_ref().and_then(|reader| self.reader_bar(reader, layout));
        [folders, list, reader].into_iter().flatten().collect()
    }

    /// A press at `col`, `row` on a bar. On the thumb it takes hold of it for
    /// a drag; on the track it pages the view toward the pointer. Returns
    /// whether a bar was there.
    pub(crate) fn press_scrollbar(&mut self, col: u16, row: u16, grid: (u16, u16)) -> bool {
        let layout = self.layout(grid.0, grid.1);
        let Some(bar) = self.scrollbars(&layout).into_iter().find(|bar| bar.holds(col, row)) else {
            return false;
        };
        let (top, size) = thumb_span(bar.first, bar.visible, bar.total, usize::from(bar.rows.height));
        let at = usize::from(row - bar.rows.y);
        if (top..top + size).contains(&at) {
            self.bar_drag = Some(BarDrag { target: bar.target, grab: u16::try_from(at - top).unwrap_or(0) });
            return true;
        }
        let page = bar.visible.saturating_sub(1).max(1);
        let first = if at < top { bar.first.saturating_sub(page) } else { bar.first + page };
        self.scroll_bar_to(&layout, &bar, first);
        true
    }

    /// The pointer moved with a thumb held: the view goes where the thumb
    /// now is. Returns whether a thumb is held.
    pub(crate) fn drag_scrollbar(&mut self, row: u16, grid: (u16, u16)) -> bool {
        let Some(drag) = self.bar_drag else {
            return false;
        };
        let layout = self.layout(grid.0, grid.1);
        let Some(bar) = self.scrollbars(&layout).into_iter().find(|bar| bar.target == drag.target) else {
            return true;
        };
        let height = usize::from(bar.rows.height);
        let (_, size) = thumb_span(bar.first, bar.visible, bar.total, height);
        let below_top = usize::from(row.saturating_sub(bar.rows.y)).saturating_sub(usize::from(drag.grab));
        let first = first_for_top(below_top, size, bar.visible, bar.total, height);
        self.scroll_bar_to(&layout, &bar, first);
        true
    }

    /// Shows `first` as the first row of the bar's pane. The selection stays.
    fn scroll_bar_to(&mut self, layout: &Layout, bar: &Bar, first: usize) {
        match bar.target {
            BarTarget::List => self.show_list_from(first, bar.visible, bar.total),
            BarTarget::Reader => self.scroll.reader = first.min(bar.total.saturating_sub(bar.visible)),
            BarTarget::Folders => {
                let rows = visible_rows(&self.mail.folders);
                let spacing = layout.sidebar.as_ref().map(|sidebar| sidebar.spacing);
                let Some(spacing) = spacing else {
                    return;
                };
                let offsets = row_offsets(&rows, spacing);
                let height = folders_height(layout);
                let wanted = folders_first_at(&offsets, first);
                self.show_folders_from(wanted.min(last_folders_first(&offsets, height)), &offsets, height);
            }
        }
    }
}

#[cfg(test)]
include!("scrollbars.tests.rs");
