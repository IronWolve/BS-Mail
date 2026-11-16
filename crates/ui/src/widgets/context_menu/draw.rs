//! A menu on screen: a bordered box over the grid, one entry per row, the
//! highlighted entry in the menu's selected style and each entry's letter
//! underlined.

use surface::{Frame, Modifier, Rect};

use super::place::{first_showing, menu_rect, rows_showing};
use super::state::Menu;
use crate::theme::roles;
use crate::widgets::help_overlay::overlaps;
use crate::widgets::{DrawContext, fill_cells, put_text};

/// Draws `menu` over `grid`.
pub fn draw_menu<A: Copy>(frame: &mut Frame, ctx: &DrawContext<'_>, grid: Rect, menu: &Menu<A>) {
    let rect = menu_rect(grid, menu);
    if rect.width < 4 || rect.height < 3 {
        return;
    }
    frame.effects.retain(|effect| !overlaps(effect.area(), rect));
    fill_cells(frame, rect, ctx.full(roles::MENU_BASE));
    draw_border(frame, ctx, rect, menu.title.as_deref());
    let room = rows_showing(rect);
    let first = first_showing(menu.cursor, room);
    for (line, (index, entry)) in menu.entries.iter().enumerate().skip(first).take(room).enumerate() {
        let y = rect.y + 1 + u16::try_from(line).unwrap_or(0);
        let row = Rect::new(rect.x + 1, y, rect.width - 2, 1);
        let selected = index == menu.cursor;
        let style = if selected { ctx.full(roles::MENU_SELECTED) } else { ctx.fg(roles::MENU_BASE) };
        if selected {
            fill_cells(frame, row, style);
        }
        put_text(frame, row, row.x + 1, y, &entry.label, style);
        if let Some(at) = entry.hotkey_at() {
            underline(frame, row.x + 1 + u16::try_from(at).unwrap_or(0), y);
        }
    }
}

fn underline(frame: &mut Frame, col: u16, row: u16) {
    if let Some(cell) = frame.cells.cell_mut((col, row)) {
        cell.modifier.insert(Modifier::UNDERLINED);
    }
}

fn draw_border(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, title: Option<&str>) {
    let border = ctx.fg(roles::MENU_BORDER);
    let set = ctx.borders;
    let (left, right) = (rect.x, rect.right() - 1);
    let (top, bottom) = (rect.y, rect.bottom() - 1);
    let put = |frame: &mut Frame, col: u16, row: u16, ch: char| {
        put_text(frame, rect, col, row, &ch.to_string(), border);
    };
    for col in left + 1..right {
        put(frame, col, top, set.horizontal);
        put(frame, col, bottom, set.horizontal);
    }
    for row in top + 1..bottom {
        put(frame, left, row, set.vertical);
        put(frame, right, row, set.vertical);
    }
    put(frame, left, top, set.top_left);
    put(frame, right, top, set.top_right);
    put(frame, left, bottom, set.bottom_left);
    put(frame, right, bottom, set.bottom_right);
    if let Some(title) = title {
        put_text(frame, rect, left + 2, top, &format!(" {title} "), ctx.fg(roles::SECTION_TITLE));
    }
}

#[cfg(test)]
include!("draw.tests.rs");
