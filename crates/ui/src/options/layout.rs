//! Where the parts of the Options screen sit, for drawing and for clicks:
//! the title, the list of pages on the left, the page on the right, one line
//! for what the program has to say and the key hint.

use surface::Rect;

use super::state::Page;

/// Narrowest and shortest grid the screen works in.
pub const MIN_COLS: u16 = 60;
pub const MIN_ROWS: u16 = 16;
/// Cells the list of pages takes.
pub const LIST_CELLS: u16 = 14;
/// Widest a page grows.
pub const MAX_PAGE_CELLS: u16 = 90;
/// Cells a label takes, with the gap before the value.
pub const LABEL_CELLS: u16 = 17;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionsRects {
    pub title: Rect,
    pub rule: Rect,
    /// One row for each page.
    pub pages: Rect,
    pub divider: Rect,
    /// The rows of the page, without the scrollbar.
    pub page: Rect,
    pub scrollbar_col: u16,
    pub message: Rect,
    pub hint: Rect,
}

/// The parts inside `area`, or `None` when it is too small.
pub fn layout(area: Rect) -> Option<OptionsRects> {
    if area.width < MIN_COLS || area.height < MIN_ROWS {
        return None;
    }
    let x = area.x + 1;
    let inner = area.width - 2;
    let body_y = area.y + 2;
    let body_rows = area.height - 4;
    let pages_rows = u16::try_from(Page::ALL.len()).unwrap_or(3).min(body_rows);
    let divider_x = x + LIST_CELLS;
    let page_x = divider_x + 3;
    let page_width = (area.right().saturating_sub(page_x + 2)).min(MAX_PAGE_CELLS);
    Some(OptionsRects {
        title: Rect::new(x, area.y, inner, 1),
        rule: Rect::new(x, area.y + 1, inner, 1),
        pages: Rect::new(x, body_y, LIST_CELLS, pages_rows),
        divider: Rect::new(divider_x, body_y, 1, body_rows),
        page: Rect::new(page_x, body_y, page_width, body_rows),
        scrollbar_col: page_x + page_width,
        message: Rect::new(x, area.bottom() - 2, inner, 1),
        hint: Rect::new(x, area.bottom() - 1, inner, 1),
    })
}

/// The page whose name is under a cell.
pub fn page_at(rects: &OptionsRects, col: u16, row: u16) -> Option<Page> {
    let pages = rects.pages;
    if col < pages.x || col >= pages.right() || row < pages.y || row >= pages.bottom() {
        return None;
    }
    Page::ALL.get(usize::from(row - pages.y)).copied()
}

/// How far a page of `total` rows can scroll in `height` rows.
pub fn max_scroll(total: usize, height: u16) -> usize {
    total.saturating_sub(usize::from(height))
}

#[cfg(test)]
include!("layout.tests.rs");
