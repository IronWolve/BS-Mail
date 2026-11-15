//! The provider list on screen: a panel under the provider row with the
//! cursor row highlighted, and a mark at the top or bottom edge when more
//! rows are out of sight.

use surface::{Frame, Rect};

use super::provider_menu::ProviderMenu;
use crate::theme::roles;
use crate::widgets::{DrawContext, fill_cells, put_text};

const MORE_ABOVE: &str = "\u{25b2}";
const MORE_BELOW: &str = "\u{25bc}";

/// Draws the list in `area`, over whatever is there.
pub fn draw_provider_menu(frame: &mut Frame, ctx: &DrawContext<'_>, area: Rect, menu: &ProviderMenu) {
    if area.width < 4 || area.height == 0 {
        return;
    }
    fill_cells(frame, area, ctx.full(roles::MENU_BASE));
    let visible = usize::from(area.height);
    let top = menu.top(visible);
    for (offset, (index, choice)) in (0u16..).zip(menu.choices().iter().enumerate().skip(top).take(visible)) {
        let row = Rect::new(area.x, area.y + offset, area.width, 1);
        let style = if index == menu.cursor() { ctx.full(roles::SIDEBAR_SELECTED) } else { ctx.full(roles::MENU_BASE) };
        fill_cells(frame, row, style);
        let label = Rect::new(row.x, row.y, row.width.saturating_sub(2), 1);
        put_text(frame, label, row.x + 1, row.y, choice.label(), style);
        let mark = if offset == 0 && top > 0 {
            MORE_ABOVE
        } else if usize::from(offset) + 1 == visible && top + visible < menu.choices().len() {
            MORE_BELOW
        } else {
            continue;
        };
        put_text(frame, row, row.right() - 2, row.y, mark, style);
    }
}
