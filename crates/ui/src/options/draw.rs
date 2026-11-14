//! The Options screen: a title, the list of pages on the left, the page on
//! the right, one line for what the program has to say and the key hint. It
//! fills the whole window, in the style of the account form.

use surface::{Frame, Rect};

use super::draw_rows::draw_rows;
use super::layout::{MIN_COLS, MIN_ROWS, OptionsRects, layout, max_scroll};
use super::pages::page_rows;
use super::state::{OptionsState, Page, Stop};
use super::view::OptionsView;
use crate::format::display_width;
use crate::theme::roles;
use crate::widgets::{DrawContext, draw_scrollbar, fill_cells, put_text};

const TITLE: &str = "Options";

/// The key hints, longest first; the first that fits the width is drawn.
const HINTS: [&str; 3] = [
    "Tab or Up/Down move   Left/Right change   Enter press   Esc close",
    "Tab move  Left/Right change  Enter press  Esc close",
    "Tab move  Enter press  Esc close",
];
const CONFIRM_HINTS: [&str; 2] = [
    "Tab or Left/Right choose   Enter press   Esc keep the account",
    "Tab choose  Enter press  Esc cancel",
];

/// The hint for a line `width` cells wide.
pub fn hint_for(width: u16, confirming: bool) -> &'static str {
    let candidates: &[&str] = if confirming { &CONFIRM_HINTS } else { &HINTS };
    let fitting = candidates.iter().find(|text| display_width(text) <= usize::from(width));
    fitting.or(candidates.last()).copied().unwrap_or("")
}

/// Draws the whole screen into `area`.
pub fn draw_options(frame: &mut Frame, ctx: &DrawContext<'_>, area: Rect, view: &OptionsView<'_>, state: &OptionsState) {
    fill_cells(frame, area, ctx.full(roles::BASE));
    frame.cursor = None;
    let Some(rects) = layout(area) else {
        draw_too_small(frame, ctx, area);
        return;
    };
    put_text(frame, rects.title, rects.title.x, rects.title.y, TITLE, ctx.fg(roles::LIST_TITLE));
    draw_rule(frame, ctx, &rects);
    draw_pages(frame, ctx, &rects, state);
    draw_page(frame, ctx, &rects, view, state);
    draw_line(frame, ctx, &rects, view);
    let hint = hint_for(rects.hint.width, state.confirming);
    put_text(frame, rects.hint, rects.hint.x, rects.hint.y, hint, ctx.fg(roles::LIST_HINT_LABEL));
}

fn draw_too_small(frame: &mut Frame, ctx: &DrawContext<'_>, area: Rect) {
    let long = format!("Options need more room: make the window larger ({MIN_COLS} x {MIN_ROWS} at least)");
    let text = if display_width(&long) <= usize::from(area.width) { long } else { "make the window larger".to_string() };
    put_text(frame, area, area.x + 1, area.y, &text, ctx.fg(roles::FOOTER_ERROR));
}

fn draw_rule(frame: &mut Frame, ctx: &DrawContext<'_>, rects: &OptionsRects) {
    let rule = ctx.borders.horizontal.to_string().repeat(usize::from(rects.rule.width));
    put_text(frame, rects.rule, rects.rule.x, rects.rule.y, &rule, ctx.fg(roles::BORDER));
    let bar = ctx.borders.vertical.to_string();
    for y in rects.divider.y..rects.divider.bottom() {
        put_text(frame, rects.divider, rects.divider.x, y, &bar, ctx.fg(roles::BORDER));
    }
}

fn draw_pages(frame: &mut Frame, ctx: &DrawContext<'_>, rects: &OptionsRects, state: &OptionsState) {
    for (offset, page) in (0u16..).zip(Page::ALL) {
        let row = Rect::new(rects.pages.x, rects.pages.y + offset, rects.pages.width, 1);
        if row.y >= rects.pages.bottom() {
            break;
        }
        let shown = page == state.page;
        let style = match (shown, state.stop == Stop::Pages) {
            (true, true) => ctx.full(roles::SIDEBAR_SELECTED),
            (true, false) => ctx.full(roles::SIDEBAR_HOVER),
            (false, _) => ctx.full(roles::SIDEBAR_ITEM),
        };
        fill_cells(frame, row, style);
        let text_style = match (shown, state.stop == Stop::Pages) {
            (true, false) => ctx.fg(roles::SIDEBAR_ACCOUNT_ACTIVE),
            _ => style,
        };
        put_text(frame, row, row.x + 1, row.y, page.label(), text_style);
    }
}

fn draw_page(frame: &mut Frame, ctx: &DrawContext<'_>, rects: &OptionsRects, view: &OptionsView<'_>, state: &OptionsState) {
    let rows = page_rows(view, state, rects.page.width);
    let scroll = state.scroll.min(max_scroll(rows.len(), rects.page.height));
    draw_rows(frame, ctx, rects.page, &rows, state, scroll);
    let strip = Rect::new(rects.scrollbar_col, rects.page.y, 1, rects.page.height);
    draw_scrollbar(frame, ctx, strip.x, strip, scroll, usize::from(rects.page.height), rows.len());
}

fn draw_line(frame: &mut Frame, ctx: &DrawContext<'_>, rects: &OptionsRects, view: &OptionsView<'_>) {
    let Some(line) = view.line else {
        return;
    };
    let role = if line.problem { roles::FOOTER_ERROR } else { roles::FOOTER_TEXT };
    put_text(frame, rects.message, rects.message.x, rects.message.y, &line.text, ctx.fg(role));
}

#[cfg(test)]
include!("draw.tests.rs");
