//! The accounts section body: one row per account, a filled dot and a bold
//! address for the selected account, a ring and a plain address for others,
//! the account's Inbox unread count at the right end of the row as a folder's
//! count is (an account whose count is not known yet shows none), and the
//! row the keyboard cursor is on drawn like a selected folder.

use model::{Account, AccountId};
use surface::{Frame, Rect};

use super::cell_text::{cells, fill_cells, put_cut, put_text};
use super::draw_context::DrawContext;
use crate::format::sidebar_count;
use crate::icons::IconId;
use crate::theme::roles;

/// Icon slot from the left edge of the body.
const ICON_AT: u16 = 1;
/// Address from the left edge of the body.
const ADDRESS_AT: u16 = 5;

/// Draws one row per account, as many as fit in `body`. `cursor` is the row
/// the keyboard is on, if any, drawn in the role `bar`.
pub fn draw_account_list(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    body: Rect,
    accounts: &[Account],
    selected: Option<&AccountId>,
    cursor: Option<usize>,
    bar: &str,
) {
    for (index, account) in accounts.iter().enumerate() {
        let Ok(dy) = u16::try_from(index) else { break };
        if dy >= body.height {
            break;
        }
        let y = body.y + dy;
        let active = selected == Some(&account.id);
        let (icon, icon_role, text_role) = if active {
            (IconId::Dot, roles::SIDEBAR_ACCOUNT_DOT, roles::SIDEBAR_ACCOUNT_ACTIVE)
        } else {
            (IconId::Ring, roles::SIDEBAR_ACCOUNT_RING, roles::SIDEBAR_ACCOUNT)
        };
        let under_cursor = cursor == Some(index);
        if under_cursor {
            fill_cells(frame, Rect::new(body.x, y, body.width, 1), ctx.full(bar));
        }
        let (icon_style, text_style) = if under_cursor {
            (ctx.fg(bar), ctx.fg(bar))
        } else {
            (ctx.fg(icon_role), ctx.fg(text_role))
        };
        put_text(frame, body, body.x + ICON_AT, y, ctx.icon(icon), icon_style);
        let x = body.x + ADDRESS_AT;
        let (count, count_x) = count_slot(ctx, account, body, x);
        let gap = u16::from(count.is_some());
        let room = count_x.saturating_sub(gap).saturating_sub(x);
        put_cut(frame, body, x, y, &account.address, room, text_style);
        if let Some(text) = count {
            let style = if under_cursor { ctx.fg(bar) } else { ctx.fg(roles::SIDEBAR_COUNT) };
            put_text(frame, body, count_x, y, &text, style);
        }
    }
}

/// The unread count and where it starts. The address always keeps at least
/// the one cell its ellipsis needs: when the count would take that, it is left
/// out instead.
fn count_slot(ctx: &DrawContext<'_>, account: &Account, body: Rect, address_x: u16) -> (Option<String>, u16) {
    let count = account.unread.and_then(|unread| sidebar_count(&ctx.config.appearance, unread));
    let count_x = count.as_ref().map_or(body.right(), |text| body.right().saturating_sub(cells(text)));
    if count_x.saturating_sub(1).saturating_sub(address_x) == 0 {
        return (None, body.right());
    }
    (count, count_x)
}

#[cfg(test)]
include!("account_list.tests.rs");
