//! The main area split into the message list and the reading pane, and the
//! rows inside each of them.

use model::config::{ListWidths, ReaderPosition};
use surface::Rect;

use super::clamp::{split_cols, split_rows};
use super::columns::{ColumnSpec, layout_columns};
use super::regions::{ListRegion, ReaderRegion, inset_h};

/// Where the split divider runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Divider {
    /// A horizontal divider on this row (reader below).
    Row(u16),
    /// A vertical divider on this column (reader right).
    Column(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainSplit {
    pub list: Rect,
    pub reader: Option<Rect>,
    pub divider: Option<Divider>,
}

/// Splits `main` by the reader position and the list share. A reader that
/// cannot get its minimum size is left out and the list takes the area.
pub fn split_main(main: Rect, position: ReaderPosition, share: u16) -> MainSplit {
    let whole = MainSplit {
        list: main,
        reader: None,
        divider: None,
    };
    match position {
        ReaderPosition::Hidden => whole,
        ReaderPosition::Below => match split_rows(main.height, share) {
            Some((list_rows, reader_rows)) => {
                let divider = main.y + list_rows;
                MainSplit {
                    list: Rect::new(main.x, main.y, main.width, list_rows),
                    reader: Some(Rect::new(main.x, divider + 1, main.width, reader_rows)),
                    divider: Some(Divider::Row(divider)),
                }
            }
            None => whole,
        },
        ReaderPosition::Right => match split_cols(main.width, share) {
            Some((list_cols, reader_cols)) => {
                let divider = main.x + list_cols;
                MainSplit {
                    list: Rect::new(main.x, main.y, list_cols, main.height),
                    reader: Some(Rect::new(divider + 1, main.y, reader_cols, main.height)),
                    divider: Some(Divider::Column(divider)),
                }
            }
            None => whole,
        },
    }
}

/// A one-row rect at `y` over the width of `text`, when `y` is inside `rect`.
fn row_in(rect: Rect, text: Rect, y: u16) -> Option<Rect> {
    (y >= rect.y && y < rect.bottom()).then(|| Rect::new(text.x, y, text.width, 1))
}

/// The list's rows: title, rule, header, then the message rows.
pub fn list_region(rect: Rect, spec: &ColumnSpec, widths: &ListWidths) -> ListRegion {
    let text = inset_h(rect, 1);
    let top = rect.y;
    let below = top.saturating_add(3);
    ListRegion {
        rect,
        text,
        title: Rect::new(text.x, top, text.width, rect.height.min(1)),
        rule: row_in(rect, text, top.saturating_add(1)),
        header: Rect::new(
            text.x,
            top.saturating_add(2),
            text.width,
            rect.height.saturating_sub(2).min(1),
        ),
        rows: Rect::new(text.x, below, text.width, rect.height.saturating_sub(3)),
        columns: layout_columns(spec, widths, text.x, text.width),
    }
}

/// The reader's rows: title, rule, `header_rows` header lines, a rule, and
/// the body below. With no header lines there is no second rule.
pub fn reader_region(rect: Rect, header_rows: u16) -> ReaderRegion {
    let text = inset_h(rect, 1);
    let top = rect.y;
    let headers_y = top.saturating_add(2);
    let after_headers = headers_y.saturating_add(header_rows);
    let rule_rows = u16::from(header_rows > 0);
    let body_y = after_headers.saturating_add(rule_rows);
    ReaderRegion {
        rect,
        text,
        title: Rect::new(text.x, top, text.width, rect.height.min(1)),
        rule: row_in(rect, text, top.saturating_add(1)),
        headers: Rect::new(
            text.x,
            headers_y.min(rect.bottom()),
            text.width,
            header_rows.min(rect.bottom().saturating_sub(headers_y)),
        ),
        header_rule: if header_rows > 0 {
            row_in(rect, text, after_headers)
        } else {
            None
        },
        body: Rect::new(
            text.x,
            body_y.min(rect.bottom()),
            text.width,
            rect.bottom().saturating_sub(body_y),
        ),
    }
}

#[cfg(test)]
include!("main_split.tests.rs");
