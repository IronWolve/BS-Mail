//! The bars that can be dragged, and the sizes a drag asks for.

use surface::{PointerShape, Rect};

use crate::layout::Layout;

/// A bar under the pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drag {
    /// Between the sidebar and the main area.
    Sidebar,
    /// Between the list and the reading pane, which sits below the list.
    SplitRows,
    /// Between the list and the reading pane, which sits right of the list.
    SplitColumns,
}

impl Drag {
    /// The pointer that says this bar can be dragged.
    pub fn shape(self) -> PointerShape {
        match self {
            Self::Sidebar | Self::SplitColumns => PointerShape::ColumnResize,
            Self::SplitRows => PointerShape::RowResize,
        }
    }
}

/// The bar at column `col`, row `row`, if there is one.
pub fn divider_at(layout: &Layout, col: u16, row: u16) -> Option<Drag> {
    if let Some(sidebar) = &layout.sidebar
        && col == sidebar.rect.right()
        && row >= sidebar.rect.y
        && row < sidebar.rect.bottom()
    {
        return Some(Drag::Sidebar);
    }
    let (list, reader) = (layout.list.as_ref()?, layout.reader.as_ref()?);
    if reader.rect.y > list.rect.y {
        let on_row = row == list.rect.bottom();
        let across = col >= list.rect.x && col < list.rect.right();
        return (on_row && across).then_some(Drag::SplitRows);
    }
    let on_col = col + 1 == reader.rect.x;
    let down = row >= list.rect.y && row < list.rect.bottom();
    (on_col && down).then_some(Drag::SplitColumns)
}

/// The cells of bar `drag`, as one cell wide or high rectangle.
pub fn divider_rect(layout: &Layout, drag: Drag) -> Option<Rect> {
    match drag {
        Drag::Sidebar => {
            let sidebar = layout.sidebar.as_ref()?.rect;
            Some(Rect::new(sidebar.right(), sidebar.y, 1, sidebar.height))
        }
        Drag::SplitRows => {
            let (list, reader) = (layout.list.as_ref()?.rect, layout.reader.as_ref()?.rect);
            (reader.y > list.y).then(|| Rect::new(list.x, list.bottom(), list.width, 1))
        }
        Drag::SplitColumns => {
            let (list, reader) = (layout.list.as_ref()?.rect, layout.reader.as_ref()?.rect);
            if reader.y > list.y {
                return None;
            }
            Some(Rect::new(reader.x.checked_sub(1)?, list.y, 1, list.height))
        }
    }
}

/// The list's share of the main area, in permille, that puts the divider at
/// `first` cells from the main area's start out of `total`. The smallest
/// share that rounds back to `first`, so a drag lands exactly where the
/// pointer is.
pub fn share_for(first: u16, total: u16) -> u16 {
    if total == 0 {
        return 0;
    }
    let wanted = u32::from(first) * 1000;
    let share = wanted.saturating_sub(500).div_ceil(u32::from(total));
    u16::try_from(share.min(1000)).unwrap_or(1000)
}

/// What a drag of `drag` to column `col`, row `row` sets, as the sidebar
/// divider column and the list share, one of them.
pub fn size_for(drag: Drag, main: Rect, col: u16, row: u16) -> DragSize {
    match drag {
        Drag::Sidebar => DragSize::SidebarWidth(col),
        Drag::SplitRows => {
            DragSize::ListShare(share_for(row.saturating_sub(main.y), main.height))
        }
        Drag::SplitColumns => {
            DragSize::ListShare(share_for(col.saturating_sub(main.x), main.width))
        }
    }
}

/// The preference a drag changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragSize {
    SidebarWidth(u16),
    ListShare(u16),
}

#[cfg(test)]
include!("drag.tests.rs");
