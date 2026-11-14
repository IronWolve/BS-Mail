//! The page that shows the sign-in link while the program waits for the user
//! to finish in their web browser: the link, and buttons to open it again,
//! copy it or give up. The program opens the browser itself when the page
//! comes up; Open and Copy are for when nothing appeared.

use surface::{Frame, Modifier, Rect};

use super::form::SetupForm;
use super::layout::{PanelRects, panel_layout};
use crate::theme::roles;
use crate::widgets::{DrawContext, fill_cells, put_text};

pub const OPEN: &str = " Open in browser ";
pub const COPY: &str = " Copy link ";
pub const CANCEL: &str = " Cancel ";
const HINT: &str = "o or Enter open again   c copy the link   Esc cancel";
const INTRO: [&str; 2] = [
    "A browser window is opening: sign in there and allow access.",
    "This page finishes by itself when you are done.",
];

/// A button on the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelChoice {
    Open,
    Copy,
    Cancel,
}

fn cells(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(0)
}

/// Where the parts of the page sit in `area`.
pub fn rects(area: Rect) -> Option<PanelRects> {
    panel_layout(area, cells(OPEN), cells(COPY), cells(CANCEL))
}

/// The button under a cell, if any.
pub fn button_at(rects: &PanelRects, col: u16, row: u16) -> Option<PanelChoice> {
    let inside = |rect: Rect| row == rect.y && col >= rect.x && col < rect.right();
    if inside(rects.open) {
        Some(PanelChoice::Open)
    } else if inside(rects.copy) {
        Some(PanelChoice::Copy)
    } else if inside(rects.cancel) {
        Some(PanelChoice::Cancel)
    } else {
        None
    }
}

/// Draws the whole screen: the page fills it.
pub fn draw_panel(frame: &mut Frame, ctx: &DrawContext<'_>, area: Rect, form: &SetupForm, link: &str) {
    fill_cells(frame, area, ctx.full(roles::BASE));
    let Some(rects) = rects(area) else {
        let text = "Window too small for sign-in (make it larger)";
        put_text(frame, area, area.x + 1, area.y, text, ctx.fg(roles::FOOTER_ERROR));
        return;
    };
    let title = "Sign in through your browser";
    put_text(frame, rects.title, rects.title.x, rects.title.y, title, ctx.fg(roles::LIST_TITLE));
    for (offset, line) in (0u16..).zip(INTRO) {
        let at = Rect::new(rects.intro.x, rects.intro.y + offset, rects.intro.width, 1);
        put_text(frame, at, at.x, at.y, line, ctx.fg(roles::READER_HEADER_LABEL));
    }
    draw_link(frame, ctx, rects.link, link);
    draw_button(frame, ctx, rects.open, OPEN, true);
    draw_button(frame, ctx, rects.copy, COPY, false);
    draw_button(frame, ctx, rects.cancel, CANCEL, false);
    let (text, style) = match (&form.notice, &form.info) {
        (Some(notice), _) => (notice.as_str(), ctx.fg(roles::FOOTER_ERROR)),
        (None, Some(info)) => (info.as_str(), ctx.fg(roles::FOOTER_TEXT)),
        (None, None) => ("Waiting for you to finish in the browser\u{2026}", ctx.fg(roles::FOOTER_TEXT)),
    };
    put_text(frame, rects.notice, rects.notice.x, rects.notice.y, text, style);
    put_text(frame, rects.hint, rects.hint.x, rects.hint.y, HINT, ctx.fg(roles::LIST_HINT_LABEL));
    frame.cursor = None;
}

/// The link cut into rows of `width` cells, with an ellipsis when it does not
/// all fit. Copying always gives the whole link.
pub fn wrap_link(link: &str, width: usize, rows: usize) -> Vec<String> {
    let chars: Vec<char> = link.chars().collect();
    let width = width.max(1);
    let mut lines: Vec<String> = chars.chunks(width).map(|chunk| chunk.iter().collect()).collect();
    if lines.len() > rows {
        lines.truncate(rows);
        if let Some(last) = lines.last_mut() {
            let keep = width.saturating_sub(1);
            *last = last.chars().take(keep).chain(std::iter::once('\u{2026}')).collect();
        }
    }
    lines
}

fn draw_link(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, link: &str) {
    let style = ctx.full(roles::MENU_BASE);
    fill_cells(frame, rect, style);
    let width = usize::from(rect.width.saturating_sub(2));
    for (offset, line) in (0u16..).zip(wrap_link(link, width, usize::from(rect.height))) {
        let row = Rect::new(rect.x, rect.y + offset, rect.width, 1);
        put_text(frame, row, row.x + 1, row.y, &line, style);
    }
}

fn draw_button(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, text: &str, primary: bool) {
    let style = if primary {
        ctx.full(roles::SIDEBAR_SELECTED).add_modifier(Modifier::BOLD)
    } else {
        ctx.full(roles::KEYBAR_CAP)
    };
    fill_cells(frame, rect, style);
    put_text(frame, rect, rect.x, rect.y, text, style);
}

#[cfg(test)]
include!("link_panel.tests.rs");
