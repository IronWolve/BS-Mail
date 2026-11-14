//! The account form on screen: a column of labelled fields in the middle of
//! the window, a Connect button, the provider's note and what went wrong.

use surface::{CursorRequest, CursorShape, Frame, Modifier, Rect};

use super::form::{SetupField, SetupForm};
use crate::composer::draw::{horizontal_skip, put_cell};
use crate::composer::graphemes::{cells_between, list};
use crate::theme::roles;
use crate::widgets::{DrawContext, fill_cells, put_text};

/// Cells the labels take, including the gap before the field.
const LABEL_CELLS: u16 = 20;
/// Widest the form grows.
const MAX_WIDTH: u16 = 84;
/// Rows the form needs: title, intro, three blocks of fields, button, notes.
pub const FORM_ROWS: u16 = 22;
/// Narrowest it works in.
pub const FORM_COLS: u16 = 56;
/// Cells a port field takes, and the label before it.
const PORT_CELLS: u16 = 6;
const PORT_LABEL_CELLS: u16 = 5;
const BUTTON: &str = " Connect and save ";
const WAITING_BUTTON: &str = " Connecting\u{2026} ";
const HINT: &str = "Tab next   Shift+Tab back   Space or arrows change a choice   F10 connect   Esc quit";

/// Where each part sits, for drawing and for clicks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupRects {
    pub title: Rect,
    pub intro: Rect,
    /// The input area of each field, in `SetupField::ALL` order.
    pub fields: [Rect; 12],
    pub note: Rect,
    pub notice: Rect,
    pub hint: Rect,
}

impl SetupRects {
    pub fn field(&self, field: SetupField) -> Rect {
        self.fields[field.index()]
    }

    /// The field under a cell, if any. A label counts with its field.
    pub fn field_at(&self, col: u16, row: u16) -> Option<SetupField> {
        SetupField::ALL.into_iter().find(|field| {
            let rect = self.field(*field);
            let from = rect.x.saturating_sub(self.label_cells(*field));
            row == rect.y && col >= from && col < rect.right()
        })
    }

    fn label_cells(&self, field: SetupField) -> u16 {
        match field {
            SetupField::ImapPort | SetupField::SmtpPort => PORT_LABEL_CELLS,
            _ => LABEL_CELLS,
        }
    }
}

/// The parts of the form inside `area`, or `None` when it is too small.
pub fn layout(area: Rect) -> Option<SetupRects> {
    if area.width < FORM_COLS || area.height < FORM_ROWS {
        return None;
    }
    let width = area.width.saturating_sub(2).min(MAX_WIDTH);
    let x = area.x + (area.width - width) / 2;
    let top = area.y + (area.height - FORM_ROWS) / 3;
    let row = |offset: u16| Rect::new(x, top + offset, width, 1);
    let field_width = width - LABEL_CELLS;
    let field = |offset: u16| Rect::new(x + LABEL_CELLS, top + offset, field_width, 1);
    let host = |offset: u16| Rect::new(x + LABEL_CELLS, top + offset, field_width - PORT_LABEL_CELLS - PORT_CELLS - 1, 1);
    let port = |offset: u16| Rect::new(x + width - PORT_CELLS, top + offset, PORT_CELLS, 1);
    let save = Rect::new(x + LABEL_CELLS, top + 16, button_cells(), 1);
    let mut fields = [Rect::default(); 12];
    let mut put = |which: SetupField, rect: Rect| fields[which.index()] = rect;
    put(SetupField::Email, field(3));
    put(SetupField::Password, field(4));
    put(SetupField::Name, field(5));
    put(SetupField::ImapHost, host(7));
    put(SetupField::ImapPort, port(7));
    put(SetupField::ImapSecurity, field(8));
    put(SetupField::SmtpHost, host(9));
    put(SetupField::SmtpPort, port(9));
    put(SetupField::SmtpSecurity, field(10));
    put(SetupField::Username, field(12));
    put(SetupField::AllowChanges, field(13));
    put(SetupField::Save, save);
    Some(SetupRects {
        title: row(0),
        intro: row(1),
        fields,
        note: Rect::new(x, top + 18, width, 2),
        notice: row(20),
        hint: Rect::new(area.x + 1, area.bottom() - 1, area.width.saturating_sub(2), 1),
    })
}

fn button_cells() -> u16 {
    u16::try_from(BUTTON.chars().count()).unwrap_or(18)
}

/// Draws the whole screen: the form fills it.
pub fn draw_setup(frame: &mut Frame, ctx: &DrawContext<'_>, area: Rect, form: &SetupForm) {
    fill_cells(frame, area, ctx.full(roles::BASE));
    let Some(rects) = layout(area) else {
        let text = format!("Window too small for setup (needs {FORM_COLS} x {FORM_ROWS})");
        put_text(frame, area, area.x + 1, area.y, &text, ctx.fg(roles::FOOTER_ERROR));
        return;
    };
    put_text(frame, rects.title, rects.title.x, rects.title.y, "Add your email account", ctx.fg(roles::LIST_TITLE));
    let intro = "Your details are kept in a file on this computer that only you can read.";
    put_text(frame, rects.intro, rects.intro.x, rects.intro.y, intro, ctx.fg(roles::READER_HEADER_LABEL));
    for field in SetupField::ALL {
        draw_field(frame, ctx, &rects, form, field);
    }
    put_text(frame, rects.note, rects.note.x, rects.note.y, form.provider_note, ctx.fg(roles::LIST_HINT_LABEL));
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
        _ if form.allow_changes => "Yes: mail can be changed and sent".to_string(),
        _ => "No: read-only, nothing is changed or sent".to_string(),
    }
}

fn draw_text(frame: &mut Frame, rect: Rect, form: &SetupForm, field: SetupField, style: surface::Style) {
    let Some(buffer) = form.buffer(field) else {
        return;
    };
    let room = usize::from(rect.width.saturating_sub(2));
    let masked = field == SetupField::Password;
    let skip = if masked { masked_skip(buffer.cursor().col, room) } else { horizontal_skip(buffer, room) };
    let inner = Rect::new(rect.x + 1, rect.y, rect.width.saturating_sub(1), 1);
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
    let text = if form.waiting { WAITING_BUTTON } else { BUTTON };
    fill_cells(frame, rect, style);
    put_text(frame, rect, rect.x, rect.y, text, style);
}

fn draw_notice(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, form: &SetupForm) {
    let (text, style) = match (&form.notice, form.waiting) {
        (Some(notice), _) => (notice.as_str(), ctx.fg(roles::FOOTER_ERROR)),
        (None, true) => ("Connecting to the server\u{2026}", ctx.fg(roles::FOOTER_TEXT)),
        (None, false) => return,
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
    let cell = if form.focus == SetupField::Password {
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
