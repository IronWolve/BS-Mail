//! The second page of the account form on screen: the rows of the settings,
//! two buttons, a note about the folders and what went wrong. It is drawn in
//! place of the first page, in the same frame, so the window and the terminal
//! show the same thing.

use surface::{CursorRequest, CursorShape, Frame, Modifier, Rect, Style};

use super::draw::{draw_notice, wrap_words};
use super::form::SetupForm;
use super::layout::{BUTTON, FORM_COLS, FORM_ROWS};
use super::settings_layout::{BACK_BUTTON, SettingsRects, settings_layout};
use super::settings_page::SettingsField;
use crate::composer::Pos;
use crate::composer::draw::put_cell;
use crate::composer::graphemes::{cells_between, count, list};
use crate::theme::roles;
use crate::widgets::{DrawContext, fill_cells, put_text};
use model::{DEFAULT_POLL_SECONDS, MIN_POLL_SECONDS};

const INTRO: &str = "Kept in this account's file. A box left empty is the program's own choice.";
const HINT: &str = "Tab next   Shift+Tab back   Space or arrows change   F10 connect   Esc back to the details";
const FOLDERS_NOTE: &str = "Folders: the path on the server, as the sidebar shows it. Empty means the program guesses from \
     the server's own marks and names.";
const WAITING_BUTTON: &str = " Connecting\u{2026} ";

pub fn draw_settings(frame: &mut Frame, ctx: &DrawContext<'_>, area: Rect, form: &SetupForm) {
    let Some(rects) = settings_layout(area) else {
        let text = format!("Window too small for setup (needs {FORM_COLS} x {FORM_ROWS})");
        put_text(frame, area, area.x + 1, area.y, &text, ctx.fg(roles::FOOTER_ERROR));
        return;
    };
    put_text(frame, rects.title, rects.title.x, rects.title.y, "Account settings", ctx.fg(roles::LIST_TITLE));
    put_text(frame, rects.intro, rects.intro.x, rects.intro.y, INTRO, ctx.fg(roles::READER_HEADER_LABEL));
    for field in SettingsField::ALL {
        draw_row(frame, ctx, &rects, form, field);
    }
    let poll_note = format!("seconds; 0 is never; less than {MIN_POLL_SECONDS} counts as {MIN_POLL_SECONDS}");
    put_text(frame, rects.poll_note, rects.poll_note.x, rects.poll_note.y, &poll_note, ctx.fg(roles::LIST_HINT_LABEL));
    for (offset, line) in (0..rects.note.height).zip(wrap_words(FOLDERS_NOTE, usize::from(rects.note.width))) {
        let row = Rect::new(rects.note.x, rects.note.y + offset, rects.note.width, 1);
        put_text(frame, row, row.x, row.y, &line, ctx.fg(roles::LIST_HINT_LABEL));
    }
    draw_notice(frame, ctx, rects.notice, form);
    put_text(frame, rects.hint, rects.hint.x, rects.hint.y, HINT, ctx.fg(roles::LIST_HINT_LABEL));
    frame.cursor = cursor_request(&rects, form);
}

fn draw_row(frame: &mut Frame, ctx: &DrawContext<'_>, rects: &SettingsRects, form: &SetupForm, field: SettingsField) {
    let rect = rects.field(field);
    let focused = form.settings.focus == field;
    if field.is_button() {
        return draw_button(frame, ctx, rect, form, field, focused);
    }
    let label_style = if focused { ctx.fg(roles::SECTION_TITLE) } else { ctx.fg(roles::READER_HEADER_LABEL) };
    let label_x = rect.x - super::layout::LABEL_CELLS;
    let clip = Rect::new(label_x, rect.y, rect.x - label_x, 1);
    put_text(frame, clip, label_x, rect.y, field.label(), label_style);
    let style = if focused { ctx.full(roles::SIDEBAR_SELECTED) } else { ctx.full(roles::MENU_BASE) };
    fill_cells(frame, rect, style);
    if field.is_choice() {
        put_text(frame, rect, rect.x + 1, rect.y, &form.settings_choice_text(field), style);
    } else if field == SettingsField::Signature {
        draw_lines(frame, rect, form, style, focused);
    } else {
        draw_line(frame, rect, form, field, style, focused);
    }
}

fn draw_button(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, form: &SetupForm, field: SettingsField, focused: bool) {
    let style = if focused { ctx.full(roles::SIDEBAR_SELECTED).add_modifier(Modifier::BOLD) } else { ctx.full(roles::KEYBAR_CAP) };
    let text = match field {
        SettingsField::Back => BACK_BUTTON,
        _ if form.waiting => WAITING_BUTTON,
        _ if form.signin == super::fields::SignIn::Browser => " Sign in and save ",
        _ => BUTTON,
    };
    fill_cells(frame, rect, style);
    put_text(frame, rect, rect.x, rect.y, text, style);
}

/// A box of one line: its text, or what it says when empty.
fn draw_line(frame: &mut Frame, rect: Rect, form: &SetupForm, field: SettingsField, style: Style, focused: bool) {
    let Some(buffer) = form.settings_buffer(field) else {
        return;
    };
    let inner = Rect::new(rect.x + 1, rect.y, rect.width.saturating_sub(1), 1);
    if buffer.is_empty() && !focused {
        let hint = if field == SettingsField::Poll { DEFAULT_POLL_SECONDS.to_string() } else { field.placeholder().to_string() };
        put_text(frame, inner, inner.x, rect.y, &hint, style.add_modifier(Modifier::DIM));
        return;
    }
    let line = &buffer.lines()[0];
    let skip = skip_for(line, buffer.cursor().col, usize::from(rect.width.saturating_sub(2)));
    let selection = buffer.selection().filter(|_| focused);
    let mut x = inner.x;
    for (offset, grapheme) in list(line).into_iter().enumerate().skip(skip) {
        let at = Pos { line: 0, col: offset };
        let selected = selection.is_some_and(|(from, to)| at >= from && at < to);
        x = put_cell(frame, inner, x, rect.y, grapheme, style, selected);
    }
}

/// The signature box: its lines from the first that keeps the cursor in view.
fn draw_lines(frame: &mut Frame, rect: Rect, form: &SetupForm, style: Style, focused: bool) {
    let buffer = &form.settings.signature;
    let inner = Rect::new(rect.x + 1, rect.y, rect.width.saturating_sub(1), rect.height);
    if buffer.is_empty() && !focused {
        let hint = SettingsField::Signature.placeholder();
        put_text(frame, inner, inner.x, rect.y, hint, style.add_modifier(Modifier::DIM));
        return;
    }
    let first = first_line(buffer.cursor().line, usize::from(rect.height));
    let skip = skip_for(&buffer.lines()[buffer.cursor().line], buffer.cursor().col, usize::from(rect.width.saturating_sub(2)));
    let selection = buffer.selection().filter(|_| focused);
    for (row, (line_at, line)) in buffer.lines().iter().enumerate().skip(first).take(usize::from(rect.height)).enumerate() {
        let y = rect.y + u16::try_from(row).unwrap_or(0);
        let mut x = inner.x;
        for (offset, grapheme) in list(line).into_iter().enumerate().skip(skip) {
            let at = Pos { line: line_at, col: offset };
            let selected = selection.is_some_and(|(from, to)| at >= from && at < to);
            x = put_cell(frame, inner, x, y, grapheme, style, selected);
        }
    }
}

/// The first line of a box `rows` high that shows `cursor_line`.
fn first_line(cursor_line: usize, rows: usize) -> usize {
    (cursor_line + 1).saturating_sub(rows)
}

/// How many graphemes of a line are scrolled off the left so the cursor stays
/// in a box `room` cells wide.
fn skip_for(line: &str, col: usize, room: usize) -> usize {
    let mut skip = 0;
    while cells_between(line, skip, col) >= room && skip < count(line) {
        skip += 1;
    }
    skip
}

fn cursor_request(rects: &SettingsRects, form: &SetupForm) -> Option<CursorRequest> {
    if form.waiting {
        return None;
    }
    let field = form.settings.focus;
    let buffer = form.settings_buffer(field)?;
    let rect = rects.field(field);
    let room = usize::from(rect.width.saturating_sub(2));
    let (line, col) = (buffer.cursor().line, buffer.cursor().col);
    let skip = skip_for(&buffer.lines()[line], col, room);
    let cell = cells_between(&buffer.lines()[line], skip, col);
    let row = u16::try_from(line - first_line(line, usize::from(rect.height))).ok()?;
    let (x, y) = (rect.x + 1 + u16::try_from(cell).ok()?, rect.y + row);
    (x < rect.right() && y < rect.bottom()).then_some(CursorRequest { col: x, row: y, shape: CursorShape::Bar })
}

#[cfg(test)]
include!("settings_draw.tests.rs");
