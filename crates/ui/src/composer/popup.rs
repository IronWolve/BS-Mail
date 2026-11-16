//! The small lists that open over the composer: the From pull-down under the
//! From line, and the matches for an address under the field being typed in.
//! Both are rows of text in a panel with the chosen row highlighted.

use surface::{Frame, Rect};

use super::draw::{ComposerRects, LABEL_CELLS};
use super::form::{ComposeForm, Field};
use super::address_list::format;
use super::graphemes::count;
use crate::theme::roles;
use crate::widgets::{DrawContext, fill_cells, put_text};

/// What is open: where it sits, its rows and the chosen one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Open {
    pub rect: Rect,
    pub rows: Vec<String>,
    pub chosen: Option<usize>,
    /// The From pull-down (a click outside it only closes it) and not the
    /// matches for an address.
    pub is_from_menu: bool,
}

/// The panel under `anchor` for `rows`, as wide as the longest row needs and
/// the room to the right allows, as tall as there are rows and the room below
/// allows; `None` when there is no room for even one row.
pub fn panel_under(anchor: Rect, rows: &[String], bounds: Rect) -> Option<Rect> {
    let y = anchor.y + 1;
    let height = u16::try_from(rows.len()).ok()?.min(bounds.bottom().saturating_sub(y));
    let longest = rows.iter().map(|row| count(row)).max().unwrap_or(0);
    let wanted = u16::try_from(longest + 2).unwrap_or(u16::MAX);
    let width = wanted.min(bounds.right().saturating_sub(anchor.x));
    (height > 0 && width >= 4).then(|| Rect::new(anchor.x, y, width, height))
}

/// What is open over the composer for `form`, in `rects`, if anything.
pub fn open_popup(form: &ComposeForm, rects: &ComposerRects, bounds: Rect) -> Option<Open> {
    if let Some(cursor) = form.from_menu {
        let rows: Vec<String> = form.senders.iter().map(|sender| sender.from.clone()).collect();
        let anchor = Rect::new(rects.from.x + LABEL_CELLS, rects.from.y, rects.from.width.saturating_sub(LABEL_CELLS), 1);
        let rect = panel_under(anchor, &rows, bounds)?;
        return Some(Open { rect, rows, chosen: Some(cursor), is_from_menu: true });
    }
    let shown = form.suggest.as_ref()?;
    let anchor = match form.focus {
        Field::To => rects.fields[0],
        Field::Cc => rects.fields[1],
        Field::Bcc => rects.fields[2],
        Field::From | Field::Subject | Field::Body => return None,
    };
    let rows: Vec<String> = shown.items.iter().map(|address| format(std::slice::from_ref(address))).collect();
    let rect = panel_under(anchor, &rows, bounds)?;
    Some(Open { rect, rows, chosen: shown.cursor, is_from_menu: false })
}

/// The row of the panel at cell (`col`, `row`), when it is on one.
pub fn row_at(open: &Open, col: u16, row: u16) -> Option<usize> {
    let rect = open.rect;
    let inside = col >= rect.x && col < rect.right() && row >= rect.y && row < rect.bottom();
    inside.then(|| usize::from(row - rect.y))
}

/// Draws the panel over whatever is there.
pub fn draw_popup(frame: &mut Frame, ctx: &DrawContext<'_>, open: &Open) {
    let area = open.rect;
    fill_cells(frame, area, ctx.full(roles::MENU_BASE));
    for (offset, text) in (0u16..).zip(&open.rows) {
        let row = Rect::new(area.x, area.y + offset, area.width, 1);
        if row.y >= area.bottom() {
            break;
        }
        let style = if open.chosen == Some(usize::from(offset)) { ctx.full(roles::SIDEBAR_SELECTED) } else { ctx.full(roles::MENU_BASE) };
        fill_cells(frame, row, style);
        let label = Rect::new(row.x, row.y, row.width.saturating_sub(1), 1);
        put_text(frame, label, row.x + 1, row.y, text, style);
    }
}

#[cfg(test)]
include!("popup.tests.rs");
