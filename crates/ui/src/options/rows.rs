//! A page as a list of rows. Each page builds its rows from the facts it is
//! given; drawing, clicking and the keyboard all read the same rows, so what
//! is drawn is exactly what can be pressed.

use model::ConnectionState;

use super::control::Control;
use crate::format::display_width;
use crate::setup::draw::wrap_words;

/// Cells between two items on one row.
pub const ITEM_GAP: u16 = 2;
/// Cells a minus or a plus button takes.
pub const STEP_CELLS: u16 = 3;

/// How read-only text is coloured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Plain,
    Dim,
    Warn,
}

/// Which end of text that is too long is cut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cut {
    /// Keep the start, end with an ellipsis.
    Right,
    /// Keep the end, start with an ellipsis (a path).
    Left,
}

/// One thing on a row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Text { text: String, tone: Tone, cut: Cut },
    /// `< value >`
    Choice { control: Control, text: String },
    /// ` - ` value ` + `
    Stepper { control: Control, text: String },
    Button { control: Control, text: String },
    /// A row of the account list: it fills the row it is on.
    Account { index: usize, address: String, state: Option<ConnectionState>, selected: bool },
}

impl Item {
    pub fn text(text: impl Into<String>, tone: Tone) -> Self {
        Self::Text { text: text.into(), tone, cut: Cut::Right }
    }

    pub fn button(control: Control, text: &str) -> Self {
        Self::Button { control, text: text.to_string() }
    }

    /// The control this item belongs to, if it is not read-only.
    pub fn control(&self) -> Option<Control> {
        match self {
            Self::Text { .. } => None,
            Self::Choice { control, .. } | Self::Stepper { control, .. } | Self::Button { control, .. } => Some(*control),
            Self::Account { .. } => Some(Control::AccountList),
        }
    }

    /// Cells the item wants on a row with `avail` cells left.
    pub fn width(&self, avail: u16) -> u16 {
        let cells = |text: &str| u16::try_from(display_width(text)).unwrap_or(u16::MAX);
        match self {
            Self::Text { text, .. } => cells(text),
            Self::Choice { text, .. } => cells(text).saturating_add(4),
            Self::Stepper { text, .. } => cells(text).saturating_add(2 * STEP_CELLS + 2),
            Self::Button { text, .. } => cells(text).saturating_add(2),
            Self::Account { .. } => avail,
        }
    }
}

/// A row: an optional label at the left, then items.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Row {
    /// `Some` puts the items in the value column, after the label (which may
    /// be empty); `None` starts them at the left edge.
    pub label: Option<String>,
    pub items: Vec<Item>,
}

impl Row {
    pub fn blank() -> Self {
        Self::default()
    }

    pub fn text(text: impl Into<String>, tone: Tone) -> Self {
        Self { label: None, items: vec![Item::text(text, tone)] }
    }

    pub fn field(label: &str, item: Item) -> Self {
        Self { label: Some(label.to_string()), items: vec![item] }
    }

    pub fn item(item: Item) -> Self {
        Self { label: None, items: vec![item] }
    }
}

/// Text broken into rows of at most `width` characters, at spaces.
pub fn paragraph(text: &str, width: u16, tone: Tone) -> Vec<Row> {
    wrap_words(text, usize::from(width.max(1))).into_iter().map(|line| Row::text(line, tone)).collect()
}

/// Buttons in as few rows as the width allows.
pub fn button_rows(buttons: Vec<Item>, width: u16) -> Vec<Row> {
    let mut rows: Vec<Row> = Vec::new();
    let mut used = 0u16;
    for button in buttons {
        let cells = button.width(width);
        let fits = used > 0 && used.saturating_add(ITEM_GAP).saturating_add(cells) <= width;
        match rows.last_mut() {
            Some(row) if fits => {
                row.items.push(button);
                used += ITEM_GAP + cells;
            }
            _ => {
                used = cells;
                rows.push(Row::item(button));
            }
        }
    }
    rows
}

/// The controls of a page in the order Tab visits them, each once.
pub fn controls_of(rows: &[Row]) -> Vec<Control> {
    let mut found: Vec<Control> = Vec::new();
    for control in rows.iter().flat_map(|row| row.items.iter()).filter_map(Item::control) {
        if !found.contains(&control) {
            found.push(control);
        }
    }
    found
}

/// The row that holds `control`; for the account list, the row of the
/// highlighted account.
pub fn row_of(rows: &[Row], control: Control, cursor: usize) -> Option<usize> {
    rows.iter().position(|row| {
        row.items.iter().any(|item| match item {
            Item::Account { index, .. } => control == Control::AccountList && *index == cursor,
            other => other.control() == Some(control),
        })
    })
}

#[cfg(test)]
include!("rows.tests.rs");
