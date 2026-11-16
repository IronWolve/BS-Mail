//! Where the parts of the account form sit, for drawing and for clicks: the
//! form itself, and the page that shows the sign-in link.

use surface::Rect;

use super::fields::{SetupField, SignIn};
use super::provider_menu::VISIBLE_ROWS;

/// Cells the labels take, including the gap before the field.
pub const LABEL_CELLS: u16 = 20;
/// Widest the form grows.
pub const MAX_WIDTH: u16 = 84;
/// Rows the form needs when it is longest (signing in through the browser):
/// title, intro, four blocks of fields, button, notes, and the key hint. The
/// provider list opens over the rows under the provider row.
pub const FORM_ROWS: u16 = 24;
/// Narrowest it works in.
pub const FORM_COLS: u16 = 56;
/// Rows the sign-in link page needs.
pub const PANEL_ROWS: u16 = 12;
/// Cells a port field takes, and the label before it.
pub const PORT_CELLS: u16 = 6;
pub const PORT_LABEL_CELLS: u16 = 5;
pub const BUTTON: &str = " Connect and save ";
/// The button that opens the second page.
pub const MORE_BUTTON: &str = " Poll, signature, folders... ";

/// Where each part of the form sits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupRects {
    pub title: Rect,
    pub intro: Rect,
    /// The input area of each field, in `SetupField::ALL` order. A row that is
    /// not shown has an empty rectangle.
    pub fields: [Rect; 17],
    /// The provider list when it is open: under the provider row, over the
    /// rows below it. Empty when the window leaves no room for it.
    pub menu: Rect,
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
            rect.width > 0 && row == rect.y && col >= from && col < rect.right()
        })
    }

    pub fn label_cells(&self, field: SetupField) -> u16 {
        match field {
            SetupField::ImapPort | SetupField::SmtpPort => PORT_LABEL_CELLS,
            _ => LABEL_CELLS,
        }
    }
}

/// The parts of the form inside `area`, or `None` when it is too small. The
/// rows that show depend on how the user signs in.
pub fn layout(area: Rect, signin: SignIn) -> Option<SetupRects> {
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
    let mut fields = [Rect::default(); 17];
    let mut put = |which: SetupField, rect: Rect| fields[which.index()] = rect;
    put(SetupField::Email, field(3));
    put(SetupField::Provider, field(4));
    put(SetupField::SignIn, field(5));
    // The last row of the block that depends on the way of signing in.
    let last = match signin {
        SignIn::Password => {
            put(SetupField::Password, field(6));
            6
        }
        SignIn::Browser => {
            put(SetupField::ClientId, field(6));
            put(SetupField::ClientSecret, field(7));
            7
        }
    };
    put(SetupField::Name, field(last + 1));
    put(SetupField::ImapHost, host(last + 3));
    put(SetupField::ImapPort, port(last + 3));
    put(SetupField::ImapSecurity, field(last + 4));
    put(SetupField::SmtpHost, host(last + 5));
    put(SetupField::SmtpPort, port(last + 5));
    put(SetupField::SmtpSecurity, field(last + 6));
    put(SetupField::Username, field(last + 8));
    put(SetupField::AllowChanges, field(last + 9));
    put(SetupField::More, Rect::new(x + LABEL_CELLS, top + last + 10, more_cells(), 1));
    put(SetupField::Save, Rect::new(x + LABEL_CELLS, top + last + 11, button_cells(), 1));
    let menu_top = top + 5;
    let room = (area.bottom() - 1).saturating_sub(menu_top);
    Some(SetupRects {
        title: row(0),
        intro: row(1),
        fields,
        menu: Rect::new(x + LABEL_CELLS, menu_top, field_width, VISIBLE_ROWS.min(room)),
        note: Rect::new(x, top + last + 13, width, 2),
        notice: row(last + 15),
        hint: hint_row(area),
    })
}

/// The whole screen as a rectangle, for a grid of `(columns, rows)`.
pub fn area_of(grid: (u16, u16)) -> Rect {
    Rect::new(0, 0, grid.0, grid.1)
}

fn button_cells() -> u16 {
    u16::try_from(BUTTON.chars().count()).unwrap_or(18)
}

fn more_cells() -> u16 {
    u16::try_from(MORE_BUTTON.chars().count()).unwrap_or(28)
}

pub(super) fn hint_row(area: Rect) -> Rect {
    Rect::new(area.x + 1, area.bottom() - 1, area.width.saturating_sub(2), 1)
}

/// Where the parts of the sign-in link page sit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelRects {
    pub title: Rect,
    pub intro: Rect,
    /// The link, wrapped over as many rows as there is room for.
    pub link: Rect,
    pub open: Rect,
    pub copy: Rect,
    pub cancel: Rect,
    pub notice: Rect,
    pub hint: Rect,
}

/// The sign-in link page inside `area`, or `None` when it is too small.
pub fn panel_layout(area: Rect, open_cells: u16, copy_cells: u16, cancel_cells: u16) -> Option<PanelRects> {
    if area.width < FORM_COLS || area.height < PANEL_ROWS {
        return None;
    }
    let width = area.width.saturating_sub(2).min(MAX_WIDTH);
    let x = area.x + (area.width - width) / 2;
    let top = area.y + 1;
    let link_rows = (area.height - 10).clamp(2, 10);
    let buttons_at = top + 4 + link_rows + 1;
    let open = Rect::new(x, buttons_at, open_cells, 1);
    let copy = Rect::new(open.right() + 2, buttons_at, copy_cells, 1);
    let cancel = Rect::new(copy.right() + 2, buttons_at, cancel_cells, 1);
    Some(PanelRects {
        title: Rect::new(x, top, width, 1),
        intro: Rect::new(x, top + 1, width, 2),
        link: Rect::new(x, top + 4, width, link_rows),
        open,
        copy,
        cancel,
        notice: Rect::new(x, buttons_at + 2, width, 1),
        hint: hint_row(area),
    })
}
