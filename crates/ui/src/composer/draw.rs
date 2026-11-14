//! The composer on screen: a header block of fields over the body, filling
//! the main area, with the caret where the next typed letter will go.

use surface::{CursorRequest, CursorShape, Frame, Modifier, Rect};

use super::editor_buffer::{EditorBuffer, Pos};
use super::form::{ComposeForm, Field, Progress};
use super::graphemes::{cells_between, count, list, width};
use super::input::Pane;
use super::wrap::wrap;
use crate::theme::roles;
use crate::widgets::{DrawContext, fill_cells, put_text};

/// Cells the field labels take, including the gap before the text.
const LABEL_CELLS: u16 = 10;
/// Rows above the body: title, from, four fields and the rule.
const HEADER_ROWS: u16 = 7;
/// The last row holds the notice, so the body needs at least this much more.
const MIN_BODY_ROWS: u16 = 3;
const HINT: &str = "Ctrl+Enter or F10 send   Esc cancel";

/// Where each part of the composer sits inside the main area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComposerRects {
    pub title: Rect,
    pub from: Rect,
    /// To, Cc, Bcc and Subject, in order.
    pub fields: [Rect; 4],
    pub rule: Rect,
    pub body: Rect,
    pub notice: Rect,
}

/// The parts of `main`, or `None` when it is too small to write in.
pub fn layout(main: Rect) -> Option<ComposerRects> {
    if main.height < HEADER_ROWS + MIN_BODY_ROWS + 1 || main.width < LABEL_CELLS + 8 {
        return None;
    }
    let text = Rect::new(main.x + 1, main.y, main.width - 2, main.height);
    let row = |offset: u16| Rect::new(text.x, text.y + offset, text.width, 1);
    let field_x = text.x + LABEL_CELLS;
    let field = |offset: u16| Rect::new(field_x, text.y + offset, text.width - LABEL_CELLS, 1);
    Some(ComposerRects {
        title: row(0),
        from: row(1),
        fields: [field(2), field(3), field(4), field(5)],
        rule: row(6),
        body: Rect::new(text.x, text.y + HEADER_ROWS, text.width, main.height - HEADER_ROWS - 1),
        notice: row(main.height - 1),
    })
}

/// The body pane's size for input to scroll and move by.
pub fn pane(main: Rect) -> Pane {
    match layout(main) {
        Some(rects) => Pane {
            width: usize::from(rects.body.width),
            rows: usize::from(rects.body.height),
        },
        None => Pane { width: 40, rows: 5 },
    }
}

/// Draws the composer into `main` and asks for the caret.
pub fn draw_composer(frame: &mut Frame, ctx: &DrawContext<'_>, main: Rect, form: &ComposeForm) {
    fill_cells(frame, main, ctx.full(roles::BASE));
    let Some(rects) = layout(main) else {
        put_text(frame, main, main.x + 1, main.y, "Window too small to write in", ctx.fg(roles::FOOTER_ERROR));
        return;
    };
    draw_title(frame, ctx, &rects, form);
    draw_from(frame, ctx, &rects, form);
    for (rect, field) in rects.fields.iter().zip([Field::To, Field::Cc, Field::Bcc, Field::Subject]) {
        draw_field(frame, ctx, *rect, field, form);
    }
    draw_rule(frame, ctx, rects.rule);
    draw_body(frame, ctx, rects.body, form);
    draw_notice(frame, ctx, rects.notice, form);
    if form.progress == Progress::Writing {
        frame.cursor = cursor_request(&rects, form);
    }
}

fn draw_title(frame: &mut Frame, ctx: &DrawContext<'_>, rects: &ComposerRects, form: &ComposeForm) {
    let title = if form.in_reply_to.is_some() { "Reply" } else { "New message" };
    put_text(frame, rects.title, rects.title.x, rects.title.y, title, ctx.fg(roles::LIST_TITLE));
    let hint_cells = u16::try_from(HINT.chars().count()).unwrap_or(0);
    let at = rects.title.right().saturating_sub(hint_cells);
    put_text(frame, rects.title, at, rects.title.y, HINT, ctx.fg(roles::LIST_HINT_LABEL));
}

fn draw_from(frame: &mut Frame, ctx: &DrawContext<'_>, rects: &ComposerRects, form: &ComposeForm) {
    let row = rects.from;
    put_text(frame, row, row.x, row.y, "From:", ctx.fg(roles::READER_HEADER_LABEL));
    put_text(frame, row, row.x + LABEL_CELLS, row.y, &form.from, ctx.fg(roles::READER_HEADER_VALUE));
}

fn draw_field(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, field: Field, form: &ComposeForm) {
    let focused = form.focus == field;
    let label_style = if focused { ctx.fg(roles::SECTION_TITLE) } else { ctx.fg(roles::READER_HEADER_LABEL) };
    let row = Rect::new(rect.x - LABEL_CELLS, rect.y, rect.width + LABEL_CELLS, 1);
    put_text(frame, row, row.x, rect.y, field.label(), label_style);
    let buffer = form.buffer(field);
    let skip = horizontal_skip(buffer, usize::from(rect.width));
    let line = &buffer.lines()[0];
    let shown: Vec<&str> = list(line).into_iter().skip(skip).collect();
    let selection = buffer.selection().filter(|_| focused);
    let mut x = rect.x;
    for (offset, grapheme) in shown.iter().enumerate() {
        let selected = is_selected(selection, Pos { line: 0, col: skip + offset });
        x = put_cell(frame, rect, x, rect.y, grapheme, ctx.fg(roles::READER_HEADER_VALUE), selected);
    }
}

/// How many graphemes of a one-line field are scrolled off the left so the
/// cursor stays in view.
fn horizontal_skip(buffer: &EditorBuffer, room: usize) -> usize {
    let line = &buffer.lines()[0];
    let col = buffer.cursor().col;
    let mut skip = 0;
    while cells_between(line, skip, col) >= room && skip < count(line) {
        skip += 1;
    }
    skip
}

fn draw_rule(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect) {
    let rule = ctx.borders.horizontal.to_string().repeat(usize::from(rect.width));
    put_text(frame, rect, rect.x, rect.y, &rule, ctx.fg(roles::BORDER_RULE));
}

fn draw_body(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, form: &ComposeForm) {
    let visual = wrap(form.body.lines(), usize::from(rect.width));
    let selection = form.body.selection().filter(|_| form.focus == Field::Body);
    let style = ctx.fg(roles::BODY_TEXT);
    for (row, piece) in visual.iter().skip(form.body_scroll).take(usize::from(rect.height)).enumerate() {
        let y = rect.y + u16::try_from(row).unwrap_or(u16::MAX);
        let graphemes = list(&form.body.lines()[piece.line]);
        let mut x = rect.x;
        for index in piece.start..piece.end {
            let selected = is_selected(selection, Pos { line: piece.line, col: index });
            x = put_cell(frame, rect, x, y, graphemes[index], style, selected);
        }
    }
}

fn draw_notice(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, form: &ComposeForm) {
    let (text, style) = match (&form.notice, form.progress) {
        (Some(notice), _) => (notice.as_str(), ctx.fg(roles::FOOTER_ERROR)),
        (None, Progress::Sending) => ("Sending\u{2026}", ctx.fg(roles::FOOTER_TEXT)),
        (None, Progress::Writing) => return,
    };
    put_text(frame, rect, rect.x, rect.y, text, style);
}

/// One grapheme at `x`, in reverse when selected. Returns the next cell.
fn put_cell(frame: &mut Frame, clip: Rect, x: u16, y: u16, grapheme: &str, style: surface::Style, selected: bool) -> u16 {
    let style = if selected { style.add_modifier(Modifier::REVERSED) } else { style };
    let next = put_text(frame, clip, x, y, grapheme, style);
    if grapheme.chars().all(char::is_whitespace) && selected {
        // A selected blank still shows, as a reversed cell.
        put_text(frame, clip, x, y, " ", style);
    }
    next.max(x.saturating_add(u16::try_from(width(grapheme)).unwrap_or(1)))
}

fn is_selected(selection: Option<(Pos, Pos)>, at: Pos) -> bool {
    selection.is_some_and(|(from, to)| at >= from && at < to)
}

fn cursor_request(rects: &ComposerRects, form: &ComposeForm) -> Option<CursorRequest> {
    let (col, row) = match form.focus {
        Field::Body => {
            let (visual_row, cell) = form.body.cursor_cell(usize::from(rects.body.width));
            let row = visual_row.checked_sub(form.body_scroll)?;
            if row >= usize::from(rects.body.height) {
                return None;
            }
            (rects.body.x + u16::try_from(cell).ok()?, rects.body.y + u16::try_from(row).ok()?)
        }
        field => {
            let rect = rects.fields[field_index(field)];
            let buffer = form.buffer(field);
            let skip = horizontal_skip(buffer, usize::from(rect.width));
            let cell = cells_between(&buffer.lines()[0], skip, buffer.cursor().col);
            (rect.x + u16::try_from(cell).ok()?, rect.y)
        }
    };
    Some(CursorRequest {
        col,
        row,
        shape: CursorShape::Bar,
    })
}

fn field_index(field: Field) -> usize {
    match field {
        Field::To => 0,
        Field::Cc => 1,
        Field::Bcc => 2,
        Field::Subject | Field::Body => 3,
    }
}

#[cfg(test)]
include!("draw.tests.rs");
