//! What the list pane says when no account is set up: where to go to add one.

use surface::{Frame, Rect};

use crate::setup::draw::wrap_words;
use crate::theme::roles;
use crate::widgets::{Align, DrawContext, put_aligned};

pub const HEADLINE: &str = "No account is set up yet.";
pub const HOW: &str = "Press F10, then choose Accounts and Add account.";

/// Draws the two sentences centred in `body`, wrapped when it is narrow.
pub fn draw_no_account_hint(frame: &mut Frame, ctx: &DrawContext<'_>, body: Rect) {
    if body.width < 4 || body.height == 0 {
        return;
    }
    let width = usize::from(body.width.saturating_sub(2));
    let mut lines: Vec<(String, bool)> = wrap_words(HEADLINE, width).into_iter().map(|line| (line, true)).collect();
    lines.push((String::new(), false));
    lines.extend(wrap_words(HOW, width).into_iter().map(|line| (line, false)));
    let count = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let top = body.y + body.height.saturating_sub(count) / 2;
    for (offset, (line, strong)) in (0u16..).zip(lines) {
        let y = top + offset;
        if y >= body.bottom() {
            break;
        }
        let style = if strong { ctx.fg(roles::SECTION_TITLE) } else { ctx.fg(roles::LIST_HINT_LABEL) };
        put_aligned(frame, body, (body.x + 1, y, body.width.saturating_sub(2)), &line, Align::Center, style);
    }
}

#[cfg(test)]
include!("empty_hint.tests.rs");
