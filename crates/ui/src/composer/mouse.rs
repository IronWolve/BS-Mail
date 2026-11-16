//! The pointer in the composer: a click focuses a field or the body and puts
//! the caret under it, a drag selects text, the wheel scrolls the body, the
//! Send and Cancel buttons act, and a click on a row of the From list or the
//! address matches takes it.

use surface::{ButtonState, InputEvent, PointerButton, Rect};

use super::buttons::{Pressed, button_at};
use super::draw::{ComposerRects, LABEL_CELLS, buttons_for, horizontal_skip, layout};
use super::form::{ComposeForm, Field, Progress};
use super::input::{Outcome, escape, escape_while_sending};
use super::popup::{Open, open_popup, row_at};
use super::pointer_moves::col_at_cell;

/// Rows the body scrolls for one notch of the wheel.
const WHEEL_ROWS: i32 = 3;

/// The four one-line fields below the From line, in the order of
/// `ComposerRects::fields`.
const LINE_FIELDS: [Field; 4] = [Field::To, Field::Cc, Field::Bcc, Field::Subject];

/// One pointer event while the composer fills `main`.
pub fn handle_pointer(form: &mut ComposeForm, event: &InputEvent, main: Rect) -> Outcome {
    let Some(rects) = layout(main) else {
        return Outcome::Ignored;
    };
    match event {
        InputEvent::Button { button: PointerButton::Left, state: ButtonState::Pressed, pos, modifiers } => {
            press(form, &rects, main, (pos.col, pos.row), modifiers.shift)
        }
        InputEvent::Button { button: PointerButton::Left, state: ButtonState::Released, .. } => {
            if form.drag.take().is_some() { Outcome::Changed } else { Outcome::Ignored }
        }
        InputEvent::PointerMoved { pos, .. } if form.drag.is_some() => drag(form, &rects, (pos.col, pos.row)),
        InputEvent::Wheel { pos, rows, .. } => wheel(form, &rects, (pos.col, pos.row), *rows),
        _ => Outcome::Ignored,
    }
}

fn inside(rect: Rect, (col, row): (u16, u16)) -> bool {
    col >= rect.x && col < rect.right() && row >= rect.y && row < rect.bottom()
}

fn press(form: &mut ComposeForm, rects: &ComposerRects, main: Rect, at: (u16, u16), extend: bool) -> Outcome {
    let buttons = buttons_for(rects, form).and_then(|found| button_at(&found, at.0, at.1));
    if form.progress == Progress::Sending {
        return if buttons == Some(Pressed::Cancel) { escape_while_sending(form) } else { Outcome::Ignored };
    }
    let open = open_popup(form, rects, main);
    if let Some(open) = &open
        && let Some(index) = row_at(open, at.0, at.1)
    {
        return take_row(form, open, index);
    }
    if open.as_ref().is_some_and(|open| open.is_from_menu) {
        // A click outside the list only closes it.
        form.from_menu = None;
        return Outcome::Changed;
    }
    match buttons {
        Some(Pressed::Send) => return Outcome::Send,
        Some(Pressed::Cancel) => return escape(form),
        None => {}
    }
    form.notice = None;
    form.discard_armed = false;
    form.close_armed = false;
    focus_and_place(form, rects, at, extend)
}

/// A click on row `index` of the open list.
fn take_row(form: &mut ComposeForm, open: &Open, index: usize) -> Outcome {
    if open.is_from_menu {
        form.from_menu = None;
        form.choose_sender(index);
    } else {
        if let Some(shown) = form.suggest.as_mut() {
            shown.cursor = Some(index);
        }
        form.accept_suggestion();
    }
    Outcome::Changed
}

/// The click goes to the field or the body under it: focus moves there and
/// the caret is put under the pointer, and a drag from here selects.
fn focus_and_place(form: &mut ComposeForm, rects: &ComposerRects, at: (u16, u16), extend: bool) -> Outcome {
    if at.1 == rects.from.y && at.0 >= rects.from.x && at.0 < rects.from.right() {
        form.focus = Field::From;
        if form.can_choose_sender() {
            form.from_menu = Some(form.sender);
        }
        return Outcome::Changed;
    }
    for (rect, field) in rects.fields.iter().zip(LINE_FIELDS) {
        let row = Rect::new(rect.x.saturating_sub(LABEL_CELLS), rect.y, rect.width + LABEL_CELLS, 1);
        if inside(row, at) {
            form.focus = field;
            place_in_line(form, field, *rect, at.0, extend);
            form.drag = Some(field);
            return Outcome::Changed;
        }
    }
    if inside(rects.body, at) {
        form.focus = Field::Body;
        let (row, x) = body_cell(form, rects.body, at);
        form.body.move_to_cell(row, x, usize::from(rects.body.width), extend);
        form.drag = Some(Field::Body);
        return Outcome::Changed;
    }
    Outcome::Ignored
}

/// The caret of a one-line field under column `col`; a click on the label,
/// left of the text, leaves it where it was.
fn place_in_line(form: &mut ComposeForm, field: Field, rect: Rect, col: u16, extend: bool) {
    if col < rect.x {
        return;
    }
    let buffer = form.buffer(field);
    let skip = horizontal_skip(buffer, usize::from(rect.width));
    let target = col_at_cell(&buffer.lines()[0], skip, usize::from(col - rect.x));
    form.focused().move_to_col(target, extend);
}

/// The screen row of the body's text (counted from its first line) and the
/// cell column that the pointer at `at` is over, kept inside the pane.
fn body_cell(form: &ComposeForm, body: Rect, at: (u16, u16)) -> (usize, usize) {
    let row = at.1.clamp(body.y, body.bottom().saturating_sub(1)) - body.y;
    let x = at.0.saturating_sub(body.x);
    (form.body_scroll + usize::from(row), usize::from(x))
}

/// The pointer moved with the button held: the selection grows to it, and a
/// body scrolls a row when the pointer is above or below it.
fn drag(form: &mut ComposeForm, rects: &ComposerRects, at: (u16, u16)) -> Outcome {
    let Some(field) = form.drag else {
        return Outcome::Ignored;
    };
    if field == Field::Body {
        scroll_towards(form, rects.body, at.1);
        let (row, x) = body_cell(form, rects.body, at);
        form.body.move_to_cell(row, x, usize::from(rects.body.width), true);
        return Outcome::Changed;
    }
    let Some(rect) = LINE_FIELDS.iter().position(|each| *each == field).and_then(|index| rects.fields.get(index)) else {
        return Outcome::Ignored;
    };
    let buffer = form.buffer(field);
    let skip = horizontal_skip(buffer, usize::from(rect.width));
    let target = col_at_cell(&buffer.lines()[0], skip, usize::from(at.0.saturating_sub(rect.x)));
    form.focused().move_to_col(target, true);
    Outcome::Changed
}

/// One row of scrolling for a pointer dragged above or below the body.
fn scroll_towards(form: &mut ComposeForm, body: Rect, row: u16) {
    if row < body.y {
        form.body_scroll = form.body_scroll.saturating_sub(1);
    } else if row >= body.bottom() {
        let last = form.body.visual_lines(usize::from(body.width)).len().saturating_sub(1);
        form.body_scroll = (form.body_scroll + 1).min(last);
    }
}

/// The wheel over the body scrolls it; the caret stays where it is.
fn wheel(form: &mut ComposeForm, rects: &ComposerRects, at: (u16, u16), rows: i16) -> Outcome {
    if rows == 0 || !inside(rects.body, at) {
        return Outcome::Ignored;
    }
    let total = form.body.visual_lines(usize::from(rects.body.width)).len();
    let most = i32::try_from(total.saturating_sub(usize::from(rects.body.height))).unwrap_or(i32::MAX);
    let now = i32::try_from(form.body_scroll).unwrap_or(i32::MAX);
    let next = now.saturating_add(i32::from(rows) * WHEEL_ROWS).clamp(0, most.max(0));
    if next == now {
        return Outcome::Ignored;
    }
    form.body_scroll = usize::try_from(next).unwrap_or(0);
    Outcome::Changed
}

#[cfg(test)]
include!("mouse.tests.rs");
