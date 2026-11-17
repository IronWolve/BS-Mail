//! Where the parts of a passphrase prompt sit: a box in the upper middle of
//! the grid holding a stack of blocks (blank rows, wrapped text, fields, a
//! notice line and rows of buttons). Drawing and clicking both read this, so
//! they cannot disagree. The start-up prompt and the Options dialog are both
//! described as a list of blocks and laid out here.

use surface::Rect;

use crate::format::{ELLIPSIS_CHAR, display_width, truncate};
use crate::setup::draw::wrap_words;

/// Widest the box grows.
pub const MAX_WIDTH: u16 = 62;
/// Narrowest grid a box is drawn in.
pub const MIN_GRID_COLS: u16 = 36;
/// Cells between two buttons.
const BUTTON_GAP: u16 = 2;

/// How a line of text is coloured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Plain,
    Dim,
    /// What went wrong: the error colour.
    Problem,
}

/// The buttons the prompts have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonId {
    Unlock,
    Forgot,
    Quit,
    /// The second step of forgetting: set the vault aside.
    SetAside,
    Back,
    /// Save a new passphrase.
    Save,
    Cancel,
    /// Switch to a key file.
    Switch,
    Keep,
}

/// Which text box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldId {
    First,
    Second,
}

/// One part of the box, from the top.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Blank,
    Text(String, Tone),
    Field(FieldId, &'static str),
    /// A notice that always takes this many rows, so the box does not change
    /// height when it comes and goes; longer text is cut with an ellipsis.
    Notice(u16, Option<(String, Tone)>),
    Buttons(Vec<(ButtonId, &'static str)>),
}

/// What a prompt is made of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    pub title: &'static str,
    pub hint: &'static str,
    pub blocks: Vec<Block>,
}

/// The cells of the placed prompt.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Placed {
    pub frame: Rect,
    /// Wrapped lines of text, with the row each sits on.
    pub lines: Vec<(Rect, String, Tone)>,
    /// Each field: its label's place and the place the dots go.
    pub fields: Vec<(FieldId, Rect, &'static str, Rect)>,
    pub buttons: Vec<(ButtonId, Rect, &'static str)>,
}

impl Placed {
    pub fn button_at(&self, col: u16, row: u16) -> Option<ButtonId> {
        self.buttons.iter().find(|(_, rect, _)| inside(*rect, col, row)).map(|(id, ..)| *id)
    }

    pub fn field_at(&self, col: u16, row: u16) -> Option<FieldId> {
        self.fields.iter().find(|(_, label, _, input)| inside(*label, col, row) || inside(*input, col, row)).map(|(id, ..)| *id)
    }

    pub fn input_of(&self, field: FieldId) -> Option<Rect> {
        self.fields.iter().find(|(id, ..)| *id == field).map(|(_, _, _, input)| *input)
    }
}

fn inside(rect: Rect, col: u16, row: u16) -> bool {
    row == rect.y && col >= rect.x && col < rect.right()
}

fn cells(text: &str) -> u16 {
    u16::try_from(display_width(text)).unwrap_or(u16::MAX)
}

/// The prompt placed in `area`, or `None` when the grid is too small for it.
pub fn place(area: Rect, prompt: &Prompt) -> Option<Placed> {
    if area.width < MIN_GRID_COLS {
        return None;
    }
    let width = MAX_WIDTH.min(area.width - 2);
    let inner = width - 4;
    let label_cells = prompt
        .blocks
        .iter()
        .filter_map(|block| match block {
            Block::Field(_, label) => Some(cells(label) + 1),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    let mut placed = Placed::default();
    let mut rows = Vec::new();
    for block in &prompt.blocks {
        stack(block, inner, &mut rows);
    }
    let height = u16::try_from(rows.len() + 2).ok().filter(|height| *height <= area.height)?;
    let frame = Rect::new(area.x + (area.width - width) / 2, area.y + (area.height - height) / 3, width, height);
    for (offset, row) in (1u16..).zip(rows) {
        put(&mut placed, row, Rect::new(frame.x + 2, frame.y + offset, inner, 1), label_cells);
    }
    placed.frame = frame;
    Some(placed)
}

/// One row of the stack, before it has a place.
enum Row {
    Blank,
    Text(String, Tone),
    Field(FieldId, &'static str),
    Buttons(Vec<(ButtonId, &'static str)>),
}

/// The rows `block` takes at `inner` cells wide.
fn stack(block: &Block, inner: u16, rows: &mut Vec<Row>) {
    match block {
        Block::Blank => rows.push(Row::Blank),
        Block::Text(text, tone) => {
            rows.extend(wrap_words(text, usize::from(inner)).into_iter().map(|line| Row::Text(line, *tone)));
        }
        Block::Field(id, label) => rows.push(Row::Field(*id, label)),
        Block::Notice(count, notice) => notice_rows(*count, notice.as_ref(), inner, rows),
        Block::Buttons(buttons) => {
            let mut line: Vec<(ButtonId, &'static str)> = Vec::new();
            let mut used = 0u16;
            for (id, label) in buttons {
                let need = cells(label) + 2;
                let fits = line.is_empty() || used + BUTTON_GAP + need <= inner;
                if !fits {
                    rows.push(Row::Buttons(std::mem::take(&mut line)));
                    used = 0;
                }
                used = if line.is_empty() { need } else { used + BUTTON_GAP + need };
                line.push((*id, label));
            }
            if !line.is_empty() {
                rows.push(Row::Buttons(line));
            }
        }
    }
}

/// `count` rows for a notice: its words, cut short when they need more, and
/// blank rows for the rest.
fn notice_rows(count: u16, notice: Option<&(String, Tone)>, inner: u16, rows: &mut Vec<Row>) {
    let mut lines: Vec<Row> = Vec::new();
    if let Some((text, tone)) = notice {
        let wrapped = wrap_words(text, usize::from(inner));
        let shown = wrapped.len().min(usize::from(count));
        for (at, line) in wrapped.iter().enumerate().take(shown) {
            let cut = at + 1 == shown && wrapped.len() > shown;
            let text = if cut { truncate(&wrapped[at..].join(" "), usize::from(inner), ELLIPSIS_CHAR) } else { line.clone() };
            lines.push(Row::Text(text, *tone));
        }
    }
    lines.resize_with(usize::from(count), || Row::Blank);
    rows.extend(lines);
}

/// Gives a row its cells. `area` is the row inside the box.
fn put(placed: &mut Placed, row: Row, area: Rect, label_cells: u16) {
    match row {
        Row::Blank => {}
        Row::Text(text, tone) => placed.lines.push((area, text, tone)),
        Row::Field(id, label) => {
            let label_rect = Rect::new(area.x, area.y, label_cells.min(area.width), 1);
            let input = Rect::new(area.x + label_rect.width, area.y, area.width.saturating_sub(label_rect.width), 1);
            placed.fields.push((id, label_rect, label, input));
        }
        Row::Buttons(buttons) => {
            let mut x = area.x;
            for (id, label) in buttons {
                let width = cells(label) + 2;
                placed.buttons.push((id, Rect::new(x, area.y, width, 1), label));
                x += width + BUTTON_GAP;
            }
        }
    }
}

#[cfg(test)]
include!("layout.tests.rs");
