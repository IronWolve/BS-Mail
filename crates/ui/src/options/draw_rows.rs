//! Drawing the rows of a page: labels, read-only text, choices, number
//! steppers, buttons and the rows of the account list.

use surface::{Frame, Modifier, Rect, Style};

use super::accounts_page::state_word;
use super::place::{items_x, slots};
use super::rows::{Cut, Item, Row, STEP_CELLS, Tone};
use super::state::{OptionsState, Stop};
use crate::format::{ELLIPSIS_CHAR, display_width, take_width, truncate};
use crate::icons::IconId;
use crate::theme::roles;
use crate::widgets::{DrawContext, fill_cells, put_text};
use model::ConnectionState;

/// Draws every row in view of `page`, starting at row `scroll`.
pub fn draw_rows(frame: &mut Frame, ctx: &DrawContext<'_>, page: Rect, rows: &[Row], state: &OptionsState, scroll: usize) {
    let focused = match state.stop {
        Stop::Control(control) => Some(control),
        Stop::Pages => None,
    };
    for (index, row) in rows.iter().enumerate().skip(scroll).take(usize::from(page.height)) {
        let y = page.y + u16::try_from(index - scroll).unwrap_or(0);
        if let Some(label) = &row.label {
            let on_row = row.items.iter().any(|item| item.control().is_some() && item.control() == focused);
            let role = if on_row { roles::SECTION_TITLE } else { roles::READER_HEADER_LABEL };
            let clip = Rect::new(page.x, y, items_x(page, row) - page.x, 1);
            put_text(frame, clip, page.x, y, label, ctx.fg(role));
        }
    }
    for slot in slots(rows, page, scroll) {
        let item = &rows[slot.row].items[slot.item];
        let on = item.control().is_some() && item.control() == focused;
        draw_item(frame, ctx, slot.rect, item, on, state.cursor);
    }
}

fn draw_item(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, item: &Item, focused: bool, cursor: usize) {
    match item {
        Item::Text { text, tone, cut } => draw_text(frame, ctx, rect, text, *tone, *cut),
        Item::Choice { text, .. } => {
            let style = field_style(ctx, focused);
            fill_cells(frame, rect, style);
            put_text(frame, rect, rect.x, rect.y, &format!("< {text} >"), style);
        }
        Item::Stepper { text, .. } => draw_stepper(frame, ctx, rect, text, focused),
        Item::Button { text, .. } => {
            let style = button_style(ctx, focused);
            fill_cells(frame, rect, style);
            put_text(frame, rect, rect.x, rect.y, &format!(" {text} "), style);
        }
        Item::Account { index, address, state, selected } => {
            draw_account(frame, ctx, rect, (address, *state, *selected), (focused, *index == cursor));
        }
    }
}

fn field_style(ctx: &DrawContext<'_>, focused: bool) -> Style {
    if focused { ctx.full(roles::SIDEBAR_SELECTED) } else { ctx.full(roles::MENU_BASE) }
}

fn button_style(ctx: &DrawContext<'_>, focused: bool) -> Style {
    if focused { ctx.full(roles::SIDEBAR_SELECTED).add_modifier(Modifier::BOLD) } else { ctx.full(roles::KEYBAR_CAP) }
}

fn draw_text(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, text: &str, tone: Tone, cut: Cut) {
    let style = match tone {
        Tone::Plain => ctx.fg(roles::BODY_TEXT),
        Tone::Dim => ctx.fg(roles::LIST_HINT_LABEL).add_modifier(Modifier::DIM),
        Tone::Warn => ctx.fg(roles::FOOTER_ERROR),
        Tone::Heading => ctx.fg(roles::SECTION_TITLE).add_modifier(Modifier::BOLD),
    };
    let shown = match cut {
        Cut::Right => truncate(text, usize::from(rect.width), ELLIPSIS_CHAR),
        Cut::Left => shorten_left(text, usize::from(rect.width)),
    };
    put_text(frame, rect, rect.x, rect.y, &shown, style);
}

/// `text` kept to `width` cells by dropping its start and putting an
/// ellipsis there, so the end of a path stays readable.
pub fn shorten_left(text: &str, width: usize) -> String {
    if display_width(text) <= width {
        return text.to_string();
    }
    if width == 0 {
        return String::new();
    }
    let keep = width - 1;
    let reversed: String = text.chars().rev().collect();
    let tail: String = take_width(&reversed, keep).chars().rev().collect();
    format!("{ELLIPSIS_CHAR}{tail}")
}

fn draw_stepper(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, text: &str, focused: bool) {
    let buttons = button_style(ctx, focused);
    let value = field_style(ctx, focused);
    let value_rect = Rect::new(rect.x + STEP_CELLS, rect.y, rect.width.saturating_sub(2 * STEP_CELLS), 1);
    let plus_at = rect.right().saturating_sub(STEP_CELLS);
    fill_cells(frame, Rect::new(rect.x, rect.y, STEP_CELLS, 1), buttons);
    put_text(frame, rect, rect.x, rect.y, " - ", buttons);
    fill_cells(frame, value_rect, value);
    put_text(frame, value_rect, value_rect.x, rect.y, &format!(" {text} "), value);
    fill_cells(frame, Rect::new(plus_at, rect.y, STEP_CELLS, 1), buttons);
    put_text(frame, rect, plus_at, rect.y, " + ", buttons);
}

/// One account row: the dot or ring of the sidebar, the address, and the
/// connection word at the right. `shown` is (address, state, selected) and
/// `on` is (the list has the keyboard, this is the highlighted row).
fn draw_account(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, shown: (&str, Option<ConnectionState>, bool), on: (bool, bool)) {
    let (address, state, selected) = shown;
    let (focused, highlighted) = on;
    let base = match (focused, highlighted) {
        (true, true) => Some(ctx.full(roles::SIDEBAR_SELECTED)),
        (false, true) => Some(ctx.full(roles::SIDEBAR_HOVER)),
        _ => None,
    };
    if let Some(style) = base {
        fill_cells(frame, rect, style);
    }
    let pick = |role: &str| if focused && highlighted { ctx.fg(roles::SIDEBAR_SELECTED) } else { ctx.fg(role) };
    let (icon, icon_role, text_role) = if selected {
        (IconId::Dot, roles::SIDEBAR_ACCOUNT_DOT, roles::SIDEBAR_ACCOUNT_ACTIVE)
    } else {
        (IconId::Ring, roles::SIDEBAR_ACCOUNT_RING, roles::SIDEBAR_ACCOUNT)
    };
    put_text(frame, rect, rect.x + 1, rect.y, ctx.icon(icon), pick(icon_role));
    let word = state_word(state);
    let word_cells = u16::try_from(display_width(word)).unwrap_or(0);
    let word_at = rect.right().saturating_sub(word_cells + 1);
    let x = rect.x + 4;
    let room = word_at.saturating_sub(x + 1);
    put_text(frame, rect, x, rect.y, &truncate(address, usize::from(room), ELLIPSIS_CHAR), pick(text_role));
    let word_role = match state {
        Some(ConnectionState::Online) => roles::STRIP_ONLINE,
        Some(ConnectionState::Connecting) => roles::STRIP_CONNECTING,
        _ => roles::STRIP_OFFLINE,
    };
    put_text(frame, rect, word_at, rect.y, word, pick(word_role));
}

#[cfg(test)]
include!("draw_rows.tests.rs");
