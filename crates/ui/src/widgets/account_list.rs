//! The accounts section body: one row per account, a filled dot and a bold
//! address for the selected account, a ring and a plain address for others.

use model::{Account, AccountId};
use surface::{Frame, Rect};

use super::cell_text::{put_cut, put_text};
use super::draw_context::DrawContext;
use crate::icons::IconId;
use crate::theme::roles;

/// Icon slot from the left edge of the body.
const ICON_AT: u16 = 1;
/// Address from the left edge of the body.
const ADDRESS_AT: u16 = 5;

/// Draws one row per account, as many as fit in `body`.
pub fn draw_account_list(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    body: Rect,
    accounts: &[Account],
    selected: Option<&AccountId>,
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
        put_text(frame, body, body.x + ICON_AT, y, ctx.icon(icon), ctx.fg(icon_role));
        let x = body.x + ADDRESS_AT;
        let room = body.right().saturating_sub(x);
        put_cut(frame, body, x, y, &account.address, room, ctx.fg(text_role));
    }
}

#[cfg(test)]
include!("account_list.tests.rs");
