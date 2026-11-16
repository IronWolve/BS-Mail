//! The message list's title row: envelope icon, `Messages — Inbox (42
//! unread)` in one run, and the key hints at the right edge.

use surface::{Frame, Rect};

use super::account_strip::MailboxSummary;
use super::cell_text::{Run, cells, put_cut, put_runs, put_text, runs_width};
use super::draw_context::DrawContext;
use crate::format::group_thousands;
use crate::icons::IconId;
use crate::theme::roles;

/// Cells from the left edge to the title text (the icon slot and a blank).
const TITLE_AT: u16 = 3;
/// Blank cells between two hints.
const HINT_GAP: u16 = 3;

/// The key part of a hint: a plain key name or an icon of the active set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HintKey {
    Text(&'static str),
    Icon(IconId),
}

/// The key hints, left to right.
const HINTS: [(HintKey, &str); 4] = [
    (HintKey::Icon(IconId::UpDown), "Navigate"),
    (HintKey::Text("Enter"), "Read"),
    (HintKey::Text("Space"), "Toggle"),
    (HintKey::Text("/"), "Search"),
];

fn title_text(mailbox: Option<&MailboxSummary>, order: Option<&str>) -> String {
    match (mailbox, order) {
        (Some(mailbox), order) => {
            let unread = group_thousands(u64::from(mailbox.unread));
            let after = order.map_or_else(String::new, |order| format!(", {order}"));
            format!("Messages \u{2014} {} ({unread} unread){after}", mailbox.name)
        }
        (None, _) => "Messages".to_string(),
    }
}

fn hint_runs(ctx: &DrawContext<'_>, key: HintKey, label: &str) -> Vec<Run> {
    let key = match key {
        HintKey::Text(text) => text,
        HintKey::Icon(id) => ctx.icon_inline(id),
    };
    vec![
        Run::new(key, ctx.fg(roles::LIST_HINT_KEY)),
        Run::new(format!(" {label}"), ctx.fg(roles::LIST_HINT_LABEL)),
    ]
}

/// Width of the hints drawn one after another with the gaps between them.
fn hints_width(hints: &[Vec<Run>]) -> u16 {
    let gaps = HINT_GAP * u16::try_from(hints.len().saturating_sub(1)).unwrap_or(0);
    hints.iter().fold(gaps, |sum, runs| sum.saturating_add(runs_width(runs)))
}

/// The hints that fit in `room` cells; the right-most go first, and none is
/// left when not even the first fits.
fn fitting_hints(ctx: &DrawContext<'_>, room: u16) -> Vec<Vec<Run>> {
    let mut hints: Vec<Vec<Run>> =
        HINTS.iter().map(|(key, label)| hint_runs(ctx, *key, label)).collect();
    while !hints.is_empty() && hints_width(&hints) > room {
        hints.pop();
    }
    hints
}

fn draw_hints(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, title_end: u16) {
    if !ctx.config.list.show_hints {
        return;
    }
    // At least one blank cell stays between the title and the hints.
    let room = rect.right().saturating_sub(title_end).saturating_sub(1);
    let hints = fitting_hints(ctx, room);
    let mut x = rect.right().saturating_sub(hints_width(&hints));
    let gap = " ".repeat(usize::from(HINT_GAP));
    for (index, runs) in hints.iter().enumerate() {
        if index > 0 {
            x = put_text(frame, rect, x, rect.y, &gap, ctx.fg(roles::LIST_HINT_LABEL));
        }
        x = put_runs(frame, rect, x, rect.y, runs);
    }
}

/// Draws the title into the one-row `rect` (the list's text width). `mailbox`
/// is the folder shown; without one the title is just `Messages`. `order`
/// says how the rows are arranged (`threads, sorted by size (loaded mail
/// only)`) and follows the unread count.
pub fn draw_list_title(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    rect: Rect,
    mailbox: Option<&MailboxSummary>,
    order: Option<&str>,
) {
    if rect.is_empty() {
        return;
    }
    put_text(frame, rect, rect.x, rect.y, ctx.icon(IconId::Mail), ctx.fg(roles::LIST_ICON));
    let title = title_text(mailbox, order);
    let title_x = rect.x.saturating_add(TITLE_AT);
    let room = rect.right().saturating_sub(title_x);
    put_cut(frame, rect, title_x, rect.y, &title, room, ctx.fg(roles::LIST_TITLE));
    draw_hints(frame, ctx, rect, title_x.saturating_add(cells(&title)));
}

#[cfg(test)]
include!("list_title.tests.rs");
