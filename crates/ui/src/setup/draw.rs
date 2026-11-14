//! The account form on screen: a column of labelled fields in the middle of
//! the window, a Connect button, the provider's note and what went wrong.
//! While the program waits for the browser, the sign-in link page is drawn
//! instead (`link_panel.rs`).

use surface::{CursorRequest, CursorShape, Frame, Modifier, Rect};

use super::fields::{SetupField, SignIn};
use super::form::SetupForm;
use super::layout::{BUTTON, FORM_COLS, FORM_ROWS, SetupRects, layout};
use super::link_panel::draw_panel;
use crate::composer::draw::{horizontal_skip, put_cell};
use crate::composer::graphemes::{cells_between, list};
use crate::theme::roles;
use crate::widgets::{DrawContext, fill_cells, put_text};

const WAITING_BUTTON: &str = " Connecting\u{2026} ";
const HINT: &str = "Tab next   Shift+Tab back   Space or arrows change a choice   F10 connect   Esc quit";

/// Draws the whole screen: the form fills it.
pub fn draw_setup(frame: &mut Frame, ctx: &DrawContext<'_>, area: Rect, form: &SetupForm) {
    if let Some(link) = &form.link {
        draw_panel(frame, ctx, area, form, link);
        return;
    }
    fill_cells(frame, area, ctx.full(roles::BASE));
    let Some(rects) = layout(area, form.signin) else {
        let text = format!("Window too small for setup (needs {FORM_COLS} x {FORM_ROWS})");
        put_text(frame, area, area.x + 1, area.y, &text, ctx.fg(roles::FOOTER_ERROR));
        return;
    };
    put_text(frame, rects.title, rects.title.x, rects.title.y, "Add your email account", ctx.fg(roles::LIST_TITLE));
    let intro = "Your details are kept in a file on this computer that only you can read.";
    put_text(frame, rects.intro, rects.intro.x, rects.intro.y, intro, ctx.fg(roles::READER_HEADER_LABEL));
    for field in SetupField::ALL.into_iter().filter(|field| form.is_shown(*field)) {
        draw_field(frame, ctx, &rects, form, field);
    }
    draw_note(frame, ctx, rects.note, form.provider_note);
    draw_notice(frame, ctx, rects.notice, form);
    let hint = Rect::new(rects.hint.x, rects.hint.y, rects.hint.width, 1);
    put_text(frame, hint, hint.x, hint.y, HINT, ctx.fg(roles::LIST_HINT_LABEL));
    frame.cursor = cursor_request(&rects, form);
}

fn draw_field(frame: &mut Frame, ctx: &DrawContext<'_>, rects: &SetupRects, form: &SetupForm, field: SetupField) {
    let rect = rects.field(field);
    let focused = form.focus == field;
    if field == SetupField::Save {
        draw_button(frame, ctx, rect, form);
        return;
    }
    let label_style = if focused { ctx.fg(roles::SECTION_TITLE) } else { ctx.fg(roles::READER_HEADER_LABEL) };
    let label_x = rect.x - rects.label_cells(field);
    let label_clip = Rect::new(label_x, rect.y, rect.right() - label_x, 1);
    put_text(frame, label_clip, label_x, rect.y, field.label(), label_style);
    let style = if focused { ctx.full(roles::SIDEBAR_SELECTED) } else { ctx.full(roles::MENU_BASE) };
    fill_cells(frame, rect, style);
    if field.is_choice() {
        put_text(frame, rect, rect.x + 1, rect.y, &choice_text(form, field), style);
    } else {
        draw_text(frame, rect, form, field, style);
    }
}

fn choice_text(form: &SetupForm, field: SetupField) -> String {
    match field {
        SetupField::ImapSecurity => format!("< {} >", form.imap_security.label()),
        SetupField::SmtpSecurity => format!("< {} >", form.smtp_security.label()),
        SetupField::SignIn => format!("< {} >", form.signin.label()),
        _ if form.allow_changes => "Yes: mail can be changed and sent".to_string(),
        _ => "No: read-only, nothing is changed or sent".to_string(),
    }
}

fn draw_text(frame: &mut Frame, rect: Rect, form: &SetupForm, field: SetupField, style: surface::Style) {
    let Some(buffer) = form.buffer(field) else {
        return;
    };
    let inner = Rect::new(rect.x + 1, rect.y, rect.width.saturating_sub(1), 1);
    if buffer.is_empty() && form.focus != field && !field.placeholder().is_empty() {
        put_text(frame, inner, inner.x, rect.y, field.placeholder(), style.add_modifier(Modifier::DIM));
        return;
    }
    let room = usize::from(rect.width.saturating_sub(2));
    let masked = field.is_secret();
    let skip = if masked { masked_skip(buffer.cursor().col, room) } else { horizontal_skip(buffer, room) };
    let selection = buffer.selection().filter(|_| form.focus == field);
    let line = &buffer.lines()[0];
    let mut x = inner.x;
    for (offset, grapheme) in list(line).into_iter().enumerate().skip(skip) {
        let shown = if masked { "\u{2022}" } else { grapheme };
        let at = super::super::composer::Pos { line: 0, col: offset };
        let selected = selection.is_some_and(|(from, to)| at >= from && at < to);
        x = put_cell(frame, inner, x, rect.y, shown, style, selected);
    }
}

/// The dots are one cell each, so the skip is a count.
fn masked_skip(col: usize, room: usize) -> usize {
    (col + 1).saturating_sub(room)
}

fn draw_button(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, form: &SetupForm) {
    let style = if form.focus == SetupField::Save {
        ctx.full(roles::SIDEBAR_SELECTED).add_modifier(Modifier::BOLD)
    } else {
        ctx.full(roles::KEYBAR_CAP)
    };
    let text = if form.waiting {
        WAITING_BUTTON
    } else if form.signin == SignIn::Browser {
        " Sign in and save "
    } else {
        BUTTON
    };
    fill_cells(frame, rect, style);
    put_text(frame, rect, rect.x, rect.y, text, style);
}

/// The provider's note, over as many of its rows as it takes.
fn draw_note(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, note: &str) {
    for (offset, line) in (0..rect.height).zip(wrap_words(note, usize::from(rect.width))) {
        let row = Rect::new(rect.x, rect.y + offset, rect.width, 1);
        put_text(frame, row, row.x, row.y, &line, ctx.fg(roles::LIST_HINT_LABEL));
    }
}

/// Text broken at spaces into lines of at most `width` characters.
pub fn wrap_words(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}

fn draw_notice(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, form: &SetupForm) {
    let (text, style) = match (&form.notice, &form.info, form.waiting) {
        (Some(notice), _, _) => (notice.as_str(), ctx.fg(roles::FOOTER_ERROR)),
        (None, Some(info), _) => (info.as_str(), ctx.fg(roles::FOOTER_TEXT)),
        (None, None, true) => ("Connecting to the server\u{2026}", ctx.fg(roles::FOOTER_TEXT)),
        (None, None, false) => return,
    };
    put_text(frame, rect, rect.x, rect.y, text, style);
}

fn cursor_request(rects: &SetupRects, form: &SetupForm) -> Option<CursorRequest> {
    if form.waiting {
        return None;
    }
    let buffer = form.buffer(form.focus)?;
    let rect = rects.field(form.focus);
    let room = usize::from(rect.width.saturating_sub(2));
    let col = buffer.cursor().col;
    let cell = if form.focus.is_secret() {
        col - masked_skip(col, room)
    } else {
        let skip = horizontal_skip(buffer, room);
        cells_between(&buffer.lines()[0], skip, col)
    };
    let x = rect.x + 1 + u16::try_from(cell).ok()?;
    (x < rect.right()).then_some(CursorRequest {
        col: x,
        row: rect.y,
        shape: CursorShape::Bar,
    })
}

#[cfg(test)]
include!("draw.tests.rs");
