//! The result of `compute_layout`: a rectangle for every region and the line
//! segments to draw between them.

use surface::Rect;

use super::columns::ColumnLayout;
use super::sidebar_layout::SidebarLayout;

/// Smallest grid the screen is drawn on; below it the caller shows a
/// one-line "window too small" message instead.
pub const MIN_COLS: u16 = 40;
pub const MIN_ROWS: u16 = 16;

/// The window's size in cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSize {
    pub cols: u16,
    pub rows: u16,
}

impl GridSize {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self { cols, rows }
    }

    /// True when the full screen can be laid out.
    pub fn is_usable(self) -> bool {
        self.cols >= MIN_COLS && self.rows >= MIN_ROWS
    }
}

/// A frame line is a border or a divider between regions; a rule is an inset
/// line under a title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Frame,
    Rule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

/// One straight line: on row `fixed` from column `from` to `to` (both
/// included), or on column `fixed` from row `from` to `to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineSpec {
    pub kind: LineKind,
    pub orientation: Orientation,
    pub fixed: u16,
    pub from: u16,
    pub to: u16,
}

impl LineSpec {
    pub fn horizontal(kind: LineKind, row: u16, from: u16, to: u16) -> Self {
        Self {
            kind,
            orientation: Orientation::Horizontal,
            fixed: row,
            from,
            to,
        }
    }

    pub fn vertical(kind: LineKind, col: u16, from: u16, to: u16) -> Self {
        Self {
            kind,
            orientation: Orientation::Vertical,
            fixed: col,
            from,
            to,
        }
    }

    /// A rule across the width of `rect`'s first row.
    pub fn rule_in(rect: Rect) -> Self {
        Self::horizontal(
            LineKind::Rule,
            rect.y,
            rect.x,
            rect.right().saturating_sub(1).max(rect.x),
        )
    }
}

/// The message list: title row, rule, column header, then the rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListRegion {
    pub rect: Rect,
    /// `rect` inset one cell left and right; text starts here.
    pub text: Rect,
    pub title: Rect,
    pub rule: Option<Rect>,
    pub header: Rect,
    pub rows: Rect,
    pub columns: ColumnLayout,
}

/// The reading pane: title row, rule, header lines, rule, body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReaderRegion {
    pub rect: Rect,
    /// `rect` inset one cell left and right.
    pub text: Rect,
    pub title: Rect,
    pub rule: Option<Rect>,
    pub headers: Rect,
    pub header_rule: Option<Rect>,
    /// The body rows over the text width; the body text sits one cell in.
    pub body: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub grid: GridSize,
    /// The grid is under the minimum: nothing else is laid out.
    pub too_small: bool,
    /// The main area, sidebar divider to right border.
    pub main: Rect,
    pub banner: Option<Rect>,
    pub strip: Option<Rect>,
    pub sidebar: Option<SidebarLayout>,
    pub list: Option<ListRegion>,
    pub reader: Option<ReaderRegion>,
    pub key_bar: Option<Rect>,
    pub footer: Option<Rect>,
    pub lines: Vec<LineSpec>,
}

impl Layout {
    /// A layout with no regions and no lines yet.
    pub fn empty(grid: GridSize) -> Self {
        Self {
            grid,
            too_small: false,
            main: Rect::default(),
            banner: None,
            strip: None,
            sidebar: None,
            list: None,
            reader: None,
            key_bar: None,
            footer: None,
            lines: Vec::new(),
        }
    }

    /// The layout of a grid under the minimum: no regions, no lines.
    pub fn too_small(grid: GridSize) -> Self {
        Self {
            too_small: true,
            ..Self::empty(grid)
        }
    }

    /// The lines of one kind.
    pub fn lines_of(&self, kind: LineKind) -> impl Iterator<Item = &LineSpec> {
        self.lines.iter().filter(move |line| line.kind == kind)
    }
}

/// `rect` narrowed by `cells` on the left and the right.
pub fn inset_h(rect: Rect, cells: u16) -> Rect {
    let width = rect.width.saturating_sub(cells.saturating_mul(2));
    Rect::new(rect.x.saturating_add(cells), rect.y, width, rect.height)
}

#[cfg(test)]
include!("regions.tests.rs");
