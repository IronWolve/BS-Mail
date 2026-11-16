//! Context menus: a right click on a message row, a folder, an account, a
//! link or the reader opens a menu at the pointer, and the Menu key and `.`
//! open the menu of the selected item. The menu is the one widget in
//! `widgets/context_menu`; what each menu holds is in `menu_tables` and what an
//! entry does is in `menu_run`. While a menu is open it takes every key and
//! click: Up and Down and Home and End move the highlight, Enter or a click
//! chooses, a letter chooses the entry it is underlined in, Escape or a click
//! outside closes it, and the pointer over an entry highlights it.

use model::{AccountId, FolderId};
use surface::{ButtonState, InputEvent, KeyEvent, LogicalKey, NamedKey, PointerButton, Rect};

use super::hit::Hit;
use super::menu_tables::{MenuAction, MenuFacts, MenuKind, build_menu};
use crate::app_view::AppView;
use crate::layout::{Layout, SectionKind};
use crate::view_state::Focus;
use crate::widgets::context_menu::{Menu, entry_at, menu_rect};
use crate::widgets::visible_rows;

/// What the open menu is about, when that is more than the selected message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuTarget {
    /// The selected message (the marked ones, for the entries that take them).
    Selected,
    Folder(FolderId),
    Account(AccountId),
    Link(String),
}

/// A menu that is open, and what it is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenMenu {
    pub menu: Menu<MenuAction>,
    pub target: MenuTarget,
}

fn area(grid: (u16, u16)) -> Rect {
    Rect::new(0, 0, grid.0, grid.1)
}

impl AppView {
    /// Whether a context menu is open.
    pub fn menu_is_open(&self) -> bool {
        self.menu.is_some()
    }

    /// The facts the words of the message entries depend on, for the messages
    /// they act on.
    fn menu_facts(&self) -> MenuFacts {
        let targets = self.target_envelopes();
        MenuFacts {
            all_seen: !targets.is_empty() && targets.iter().all(|envelope| envelope.flags.seen),
            all_flagged: !targets.is_empty() && targets.iter().all(|envelope| envelope.flags.flagged),
            all_alert: !targets.is_empty() && targets.iter().all(|envelope| envelope.flags.alert),
            marked: self.marked.len(),
            folder: None,
        }
    }

    fn open_menu(&mut self, kind: MenuKind, target: MenuTarget, anchor: (u16, u16)) {
        let facts = match &target {
            MenuTarget::Selected => self.menu_facts(),
            MenuTarget::Folder(id) => MenuFacts { folder: self.mail.folder(*id).map(|folder| folder.role), ..MenuFacts::default() },
            _ => MenuFacts::default(),
        };
        self.menu = Some(OpenMenu { menu: build_menu(kind, &facts, anchor), target });
    }

    /// A right press at `col`, `row`: the menu of what is there, with that
    /// message selected when it is a message. Returns whether anything changed.
    pub(crate) fn right_click(&mut self, col: u16, row: u16, grid: (u16, u16)) -> bool {
        let layout = self.layout(grid.0, grid.1);
        let anchor = (col, row);
        match self.hit(&layout, col, row) {
            Hit::Message(index) => {
                self.account_cursor = None;
                self.focus = Focus::List;
                self.select_message(index, &layout);
                self.open_menu(MenuKind::Message, MenuTarget::Selected, anchor);
            }
            Hit::Folder(index) => {
                let Some(id) = visible_rows(&self.mail.folders).get(index).map(|row| row.folder.id) else {
                    return false;
                };
                self.open_menu(MenuKind::Folder, MenuTarget::Folder(id), anchor);
            }
            Hit::Account(index) => {
                let Some(id) = self.mail.accounts.get(index).map(|account| account.id.clone()) else {
                    return false;
                };
                self.open_menu(MenuKind::Account, MenuTarget::Account(id), anchor);
            }
            Hit::Reader => match self.link_under(&layout, col, row) {
                Some(link) => self.open_menu(MenuKind::Link, MenuTarget::Link(link), anchor),
                None if self.has_selected_message() => self.open_menu(MenuKind::Reader, MenuTarget::Selected, anchor),
                None => return false,
            },
            _ => return false,
        }
        true
    }

    fn has_selected_message(&self) -> bool {
        self.shown_page().is_some_and(|page| self.selection.message_index < page.envelopes.len())
    }

    /// The Menu key and `.`: the menu of the selected item of the pane that
    /// has focus, opened beside that item. Returns whether one opened.
    pub(crate) fn open_menu_for_focus(&mut self, grid: (u16, u16)) -> bool {
        let layout = self.layout(grid.0, grid.1);
        match self.focus {
            Focus::Sidebar => self.open_sidebar_menu(&layout),
            Focus::List => {
                let Some(at) = self.selected_row_cell(&layout).filter(|_| self.has_selected_message()) else {
                    return false;
                };
                self.open_menu(MenuKind::Message, MenuTarget::Selected, at);
                true
            }
            Focus::Reader => {
                let (Some(reader), true) = (layout.reader.as_ref(), self.has_selected_message()) else {
                    return false;
                };
                self.open_menu(MenuKind::Reader, MenuTarget::Selected, (reader.body.x + 2, reader.body.y));
                true
            }
        }
    }

    /// The folder or account the sidebar's cursor is on.
    fn open_sidebar_menu(&mut self, layout: &Layout) -> bool {
        if let Some(index) = self.account_cursor_row() {
            let (Some(id), Some(body)) = (
                self.mail.accounts.get(index).map(|account| account.id.clone()),
                layout.sidebar.as_ref().and_then(|sidebar| sidebar.section(SectionKind::Accounts)).map(|section| section.body),
            ) else {
                return false;
            };
            let at = (body.x + 4, body.y + u16::try_from(index).unwrap_or(0));
            self.open_menu(MenuKind::Account, MenuTarget::Account(id), at);
            return true;
        }
        let (Some(folder), Some(at)) = (self.selection.folder, self.selected_folder_cell(layout)) else {
            return false;
        };
        self.open_menu(MenuKind::Folder, MenuTarget::Folder(folder), at);
        true
    }

    /// The cell just inside the selected message's row, when it shows.
    fn selected_row_cell(&self, layout: &Layout) -> Option<(u16, u16)> {
        let row = self.message_row_rect(layout, self.selection.message_index)?;
        Some((row.x + 3, row.y))
    }

    /// The cell just inside the selected folder's row, when it shows.
    fn selected_folder_cell(&self, layout: &Layout) -> Option<(u16, u16)> {
        let at = visible_rows(&self.mail.folders).iter().position(|row| Some(row.folder.id) == self.selection.folder)?;
        let row = self.folder_row_rect(layout, at)?;
        Some((row.x + 4, row.y))
    }

    /// Input while a menu is open: all of it is the menu's. Returns whether
    /// the screen changed.
    pub(crate) fn menu_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        match event {
            InputEvent::Key(key) => self.menu_key(key, grid),
            InputEvent::PointerMoved { pos, .. } => self.menu_hover(pos.col, pos.row, grid),
            InputEvent::Button { button, state: ButtonState::Pressed, pos, .. } => {
                self.menu_press(*button, (pos.col, pos.row), grid)
            }
            InputEvent::Wheel { rows, .. } if *rows != 0 => {
                if let Some(open) = self.menu.as_mut() {
                    open.menu.step(*rows > 0);
                }
                true
            }
            _ => false,
        }
    }

    fn menu_key(&mut self, key: &KeyEvent, grid: (u16, u16)) -> bool {
        let held = key.modifiers;
        if held.ctrl || held.alt || held.meta {
            return false;
        }
        let Some(open) = self.menu.as_mut() else {
            return false;
        };
        let menu = &mut open.menu;
        match key.key {
            LogicalKey::Named(NamedKey::Escape | NamedKey::Menu) | LogicalKey::Char('.') => self.menu = None,
            LogicalKey::Named(NamedKey::Up) => menu.step(false),
            LogicalKey::Named(NamedKey::Down) => menu.step(true),
            LogicalKey::Named(NamedKey::Home) => menu.first(),
            LogicalKey::Named(NamedKey::End) => menu.last(),
            LogicalKey::Named(NamedKey::Enter) => {
                let at = menu.cursor;
                self.run_menu_entry(at, grid);
            }
            LogicalKey::Char(letter) => match menu.by_hotkey(letter) {
                Some(at) => self.run_menu_entry(at, grid),
                None => return false,
            },
            _ => return false,
        }
        true
    }

    /// The pointer over an entry highlights it.
    fn menu_hover(&mut self, col: u16, row: u16, grid: (u16, u16)) -> bool {
        let Some(open) = self.menu.as_mut() else {
            return false;
        };
        let rect = menu_rect(area(grid), &open.menu);
        match entry_at(rect, &open.menu, col, row) {
            Some(index) if index != open.menu.cursor => {
                open.menu.cursor = index;
                true
            }
            _ => false,
        }
    }

    /// A press: on an entry it chooses it; inside the box elsewhere nothing
    /// happens; outside the box it closes the menu, and a right press there
    /// opens the menu of whatever is under it.
    fn menu_press(&mut self, button: PointerButton, (col, row): (u16, u16), grid: (u16, u16)) -> bool {
        let Some(open) = self.menu.as_ref() else {
            return false;
        };
        let rect = menu_rect(area(grid), &open.menu);
        if let Some(index) = entry_at(rect, &open.menu, col, row) {
            if matches!(button, PointerButton::Left | PointerButton::Right) {
                self.run_menu_entry(index, grid);
            }
            return true;
        }
        let inside = col >= rect.x && col < rect.right() && row >= rect.y && row < rect.bottom();
        if inside {
            return true;
        }
        self.menu = None;
        if button == PointerButton::Right {
            self.right_click(col, row, grid);
        }
        true
    }

    /// The pointer over an entry of the open menu, for the hand.
    pub(crate) fn menu_entry_under(&self, col: u16, row: u16, grid: (u16, u16)) -> bool {
        self.menu.as_ref().is_some_and(|open| {
            let rect = menu_rect(area(grid), &open.menu);
            entry_at(rect, &open.menu, col, row).is_some()
        })
    }
}

#[cfg(test)]
include!("menus.tests.rs");
