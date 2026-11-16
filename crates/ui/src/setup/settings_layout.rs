//! Where the parts of the second page sit, for drawing and for clicks. The
//! page is as high as the first one at its longest, so it fits wherever the
//! form does.

use surface::Rect;

use super::layout::{BUTTON, FORM_COLS, FORM_ROWS, LABEL_CELLS, MAX_WIDTH, hint_row};
use super::settings_page::SettingsField;

/// Rows the signature box has.
pub const SIGNATURE_ROWS: u16 = 3;
/// Cells the poll box has.
pub const POLL_CELLS: u16 = 8;
/// The button that goes back to the first page.
pub const BACK_BUTTON: &str = " < Details ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsRects {
    pub title: Rect,
    pub intro: Rect,
    /// The input area of each row, in `SettingsField::ALL` order.
    pub fields: [Rect; 13],
    /// What the poll row says after its box.
    pub poll_note: Rect,
    pub notice: Rect,
    /// The words about the folders, over two rows.
    pub note: Rect,
    pub hint: Rect,
}

impl SettingsRects {
    pub fn field(&self, field: SettingsField) -> Rect {
        self.fields[field.index()]
    }

    /// The row under a cell, if any. A label counts with its row, and the
    /// signature box with all its rows.
    pub fn field_at(&self, col: u16, row: u16) -> Option<SettingsField> {
        SettingsField::ALL.into_iter().find(|field| {
            let rect = self.field(*field);
            let from = if field.is_button() { rect.x } else { rect.x.saturating_sub(LABEL_CELLS) };
            rect.width > 0 && row >= rect.y && row < rect.bottom() && col >= from && col < rect.right()
        })
    }
}

/// The parts of the page inside `area`, or `None` when it is too small.
pub fn settings_layout(area: Rect) -> Option<SettingsRects> {
    if area.width < FORM_COLS || area.height < FORM_ROWS {
        return None;
    }
    let width = area.width.saturating_sub(2).min(MAX_WIDTH);
    let x = area.x + (area.width - width) / 2;
    let top = area.y + (area.height - FORM_ROWS) / 3;
    let row = |offset: u16| Rect::new(x, top + offset, width, 1);
    let field = |offset: u16, rows: u16| Rect::new(x + LABEL_CELLS, top + offset, width - LABEL_CELLS, rows);
    let mut fields = [Rect::default(); 13];
    let mut put = |which: SettingsField, rect: Rect| fields[which.index()] = rect;
    put(SettingsField::Poll, Rect::new(x + LABEL_CELLS, top + 3, POLL_CELLS, 1));
    put(SettingsField::SentCopy, field(4, 1));
    put(SettingsField::ImapTrust, field(5, 1));
    put(SettingsField::SmtpTrust, field(6, 1));
    put(SettingsField::AllowChanges, field(7, 1));
    put(SettingsField::Signature, field(9, SIGNATURE_ROWS));
    for (at, which) in SettingsField::FOLDERS.into_iter().enumerate() {
        put(which, field(13 + u16::try_from(at).unwrap_or(0), 1));
    }
    let back = Rect::new(x + LABEL_CELLS, top + 19, cells(BACK_BUTTON), 1);
    put(SettingsField::Back, back);
    put(SettingsField::Save, Rect::new(back.right() + 2, back.y, cells(BUTTON), 1));
    let poll_x = x + LABEL_CELLS + POLL_CELLS + 2;
    Some(SettingsRects {
        title: row(0),
        intro: row(1),
        fields,
        poll_note: Rect::new(poll_x, top + 3, (x + width).saturating_sub(poll_x), 1),
        notice: row(20),
        note: Rect::new(x, top + 21, width, 2),
        hint: hint_row(area),
    })
}

fn cells(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(20)
}

#[cfg(test)]
include!("settings_layout.tests.rs");
