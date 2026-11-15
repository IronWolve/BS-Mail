//! The accounts section body: one row per account, a filled dot and a bold
//! address for the selected account, a ring and a plain address for others,
//! and the row the keyboard cursor is on drawn like a selected folder.

use model::{Account, AccountId};
use surface::{Frame, Rect};

use super::cell_text::{fill_cells, put_cut, put_text};
use super::draw_context::DrawContext;
use crate::icons::IconId;
use crate::theme::roles;

/// Icon slot from the left edge of the body.
const ICON_AT: u16 = 1;
/// Address from the left edge of the body.
const ADDRESS_AT: u16 = 5;

/// Draws one row per account, as many as fit in `body`. `cursor` is the row
/// the keyboard is on, if any.
pub fn draw_account_list(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    body: Rect,
    accounts: &[Account],
    selected: Option<&AccountId>,
    cursor: Option<usize>,
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
            fill_cells(frame, Rect::new(body.x, y, body.width, 1), ctx.full(roles::SIDEBAR_SELECTED));
        }
        let (icon_style, text_style) = if under_cursor {
            (ctx.fg(roles::SIDEBAR_SELECTED), ctx.fg(roles::SIDEBAR_SELECTED))
        } else {
            (ctx.fg(icon_role), ctx.fg(text_role))
        };
        put_text(frame, body, body.x + ICON_AT, y, ctx.icon(icon), icon_style);
        let x = body.x + ADDRESS_AT;
        let room = body.right().saturating_sub(x);
        put_cut(frame, body, x, y, &account.address, room, text_style);
    }
}

#[cfg(test)]
include!("account_list.tests.rs");
