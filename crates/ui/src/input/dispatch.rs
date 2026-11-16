//! Input events into changes of selection, focus and scroll.
//!
//! Nothing here talks to the backend. Selection moves; the commands the new
//! selection needs are worked out afterwards by `AppView::take_commands`.

use model::Command;
use surface::{ButtonState, InputEvent, KeyEvent, LogicalKey, Modifiers, NamedKey};

use super::action::{Action, action_for};
use super::compose::ComposeKind;
use super::hit::{Hit, HitView, hit_at};
use super::motion::folders_height;
use super::movement::WHEEL_ROWS;
use crate::app_view::AppView;
use crate::layout::Layout;
use crate::view_state::{Focus, ListPage, OverlayKind};
use crate::widgets::{row_offsets, visible_rows};

impl AppView {
    /// Reacts to one input event on a `cols` x `rows` grid, once the text a
    /// command key brought with it has been set aside (see `paired_text`).
    pub(crate) fn dispatch_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        if self.setup.is_none() && self.options.is_none() && self.layout(grid.0, grid.1).too_small {
            return self.input_while_hidden(event, grid);
        }
        if self.setup.is_some() {
            return self.setup_input(event, grid);
        }
        if self.options.is_some() {
            return self.options_input(event, grid);
        }
        if self.composer.is_some() {
            return self.composer_input(event, grid);
        }
        if self.delete_prompt.is_some() {
            return self.delete_prompt_input(event, grid);
        }
        if self.picker.is_some() {
            return self.picker_input(event, grid);
        }
        if self.search_bar.is_some() {
            return self.search_input(event, grid);
        }
        if self.menu.is_some() {
            return self.menu_input(event, grid);
        }
        if self.overlays.is_open() {
            return self.input_over_overlay(event, grid);
        }
        match event {
            InputEvent::Key(key) => self.main_key(key, grid),
            _ => self.pointer_event(event, grid),
        }
    }

    /// A key on the main screen. A drag of rows that is going ends with it,
    /// and Escape only ends that; the letter after `g` is the folder's.
    fn main_key(&mut self, key: &KeyEvent, grid: (u16, u16)) -> bool {
        let was_dragging = self.dragging_rows();
        self.cancel_row_drag();
        if was_dragging && key.key == LogicalKey::Named(NamedKey::Escape) {
            return true;
        }
        if self.memory.go_pending {
            return self.go_key(key);
        }
        match action_for(key) {
            Some(action) => {
                self.act(action, grid);
                true
            }
            None => was_dragging,
        }
    }

    /// While a screen is open over this one, a key press or a click closes
    /// it and nothing else reaches the screen underneath. Help that is cut
    /// short on a small grid first takes the keys that scroll it.
    fn input_over_overlay(&mut self, event: &InputEvent, grid: (u16, u16)) -> bool {
        if self.help_scrolls(event, grid) {
            return true;
        }
        let closes = matches!(
            event,
            InputEvent::Key(_)
                | InputEvent::Button {
                    state: ButtonState::Pressed,
                    ..
                }
        );
        if closes {
            self.overlays.open = None;
            self.overlays.help_scroll = 0;
        }
        closes
    }

    /// The commands the backend is due, in order. Call after every input
    /// event and every backend event.
    pub fn take_commands(&mut self) -> Vec<Command> {
        let mut commands = self.queue.take(&self.selection, &self.mail);
        self.note_own_changes(&commands);
        // Older mail is only asked for while the folder's own list is shown,
        // not while the results of a search are.
        if self.search.is_none() {
            commands.extend(self.queue.take_more(&self.selection, &self.mail, self.scroll.list_reach));
        }
        commands
    }

    /// Asks the backend to look at the server again, for the open folder and
    /// message too.
    pub fn refresh(&mut self) {
        self.queue.forget();
        if let Some(account) = self.selection.account.clone() {
            self.queue.push(Command::SyncNow { account });
        }
    }

    pub(crate) fn act(&mut self, action: Action, grid: (u16, u16)) {
        if self.account_cursor.is_some() && self.account_cursor_key(action, grid) {
            return;
        }
        if self.act_on_messages(action, grid) {
            return;
        }
        match action {
            Action::FocusNext => self.change_focus(self.focus.next()),
            Action::FocusPrevious => self.change_focus(self.focus.previous()),
            Action::Enter if self.focus == Focus::List && self.selected_is_draft() => {
                self.start_compose(ComposeKind::Draft);
            }
            Action::Enter => self.focus = deeper(self.focus),
            Action::Back if self.focus == Focus::List && !self.marked.is_empty() => self.clear_marks(),
            Action::Back if self.focus == Focus::List && self.search.is_some() => self.end_search(),
            Action::Back => self.focus = shallower(self.focus),
            Action::Right if self.focus == Focus::Sidebar => self.open_folder_or_go_in(),
            Action::Right => self.focus = deeper(self.focus),
            Action::Left if self.focus == Focus::Sidebar => self.close_folder_or_go_up(),
            Action::Left => self.focus = shallower(self.focus),
            Action::SpaceDown if self.focus == Focus::Reader => {
                self.act(Action::PageDown, grid);
            }
            Action::SpaceUp if self.focus == Focus::Reader => self.act(Action::PageUp, grid),
            Action::SpaceDown if self.focus == Focus::List => self.mark_and_move(Action::Down, grid),
            Action::SpaceUp if self.focus == Focus::List => self.mark_and_move(Action::Up, grid),
            Action::SpaceDown | Action::SpaceUp => {}
            Action::Sync => self.refresh(),
            Action::Help => {
                self.overlays.open = Some(OverlayKind::Help);
                self.overlays.help_scroll = 0;
            }
            Action::Options => self.open_options(),
            Action::Search => self.start_search(),
            Action::Compose => self.start_compose(ComposeKind::New),
            Action::Reply => self.start_compose(ComposeKind::Reply),
            Action::ReplyAll => self.start_compose(ComposeKind::ReplyAll),
            Action::Forward => self.start_compose(ComposeKind::Forward),
            Action::EditAsNew => self.start_compose(ComposeKind::EditAsNew),
            Action::FocusPane(focus) => self.change_focus(focus),
            Action::ContextMenu => {
                self.open_menu_for_focus(grid);
            }
            Action::GoTo => self.begin_go_to(),
            movement => {
                let layout = self.layout(grid.0, grid.1);
                match self.focus {
                    Focus::Sidebar => {
                        if !self.run_on_into_accounts(movement, &layout) {
                            self.move_folder(movement, &layout);
                        }
                    }
                    Focus::List => self.move_message(movement, &layout),
                    Focus::Reader => self.scroll_reader(movement, &layout),
                }
            }
        }
    }

    /// Moves focus to another pane; the sidebar's account cursor stays behind.
    fn change_focus(&mut self, focus: Focus) {
        self.account_cursor = None;
        self.focus = focus;
    }

    /// Space in the list: marks the message and moves on, so a run of Space
    /// marks a run of messages.
    fn mark_and_move(&mut self, direction: Action, grid: (u16, u16)) {
        self.toggle_mark();
        let layout = self.layout(grid.0, grid.1);
        self.move_message(direction, &layout);
    }

    pub(super) fn click(&mut self, col: u16, row: u16, modifiers: Modifiers, grid: (u16, u16)) -> bool {
        self.account_cursor = None;
        self.cancel_go_to();
        let layout = self.layout(grid.0, grid.1);
        match self.hit(&layout, col, row) {
            Hit::Folder(index) => {
                self.focus = Focus::Sidebar;
                self.select_folder_row(index);
            }
            Hit::Message(index) => self.press_message(index, (col, row), modifiers, &layout),
            Hit::Account(index) => return self.click_account(index),
            Hit::Sidebar => self.focus = Focus::Sidebar,
            Hit::ListChrome => self.focus = Focus::List,
            Hit::Reader => self.click_reader(&layout, col, row),
            Hit::KeyBar(action) => self.press_button(action, grid),
            Hit::Nothing => return self.footer_span_at(&layout, col, row).is_some_and(|span| self.click_footer(span)),
        }
        true
    }

    /// A click on a key bar button does what its key does.
    pub(super) fn press_button(&mut self, action: &str, grid: (u16, u16)) {
        match action {
            "compose" => self.act(Action::Compose, grid),
            "reply" => self.act(Action::Reply, grid),
            "reply_all" => self.act(Action::ReplyAll, grid),
            "forward" => self.act(Action::Forward, grid),
            "edit_as_new" => self.act(Action::EditAsNew, grid),
            "delete" => self.act(Action::Delete, grid),
            "archive" => self.act(Action::Archive, grid),
            "spam" => self.act(Action::Spam, grid),
            "mark_all" => self.act(Action::MarkAll, grid),
            "unmark_all" => self.act(Action::UnmarkAll, grid),
            "invert_marks" => self.act(Action::InvertMarks, grid),
            "next_unread" => self.act(Action::NextUnread, grid),
            "move" => self.act(Action::Move, grid),
            "search" => self.act(Action::Search, grid),
            "sync" => self.act(Action::Sync, grid),
            "help" => self.act(Action::Help, grid),
            "options" => self.act(Action::Options, grid),
            "quit" => self.quit_requested = true,
            other => self.mail.set_note(Some(format!("{other} is not built yet"))),
        }
    }

    pub(super) fn wheel(&mut self, col: u16, row: u16, rows: i16, grid: (u16, u16)) -> bool {
        if rows == 0 {
            return false;
        }
        let layout = self.layout(grid.0, grid.1);
        let target = match self.hit(&layout, col, row) {
            Hit::Folder(_) | Hit::Account(_) | Hit::Sidebar => Focus::Sidebar,
            Hit::Message(_) | Hit::ListChrome => Focus::List,
            Hit::Reader => Focus::Reader,
            Hit::KeyBar(_) | Hit::Nothing => return false,
        };
        // The pane under the pointer scrolls; the selection and the focus stay.
        match target {
            Focus::Sidebar => self.wheel_folders(&layout, rows),
            Focus::List => self.wheel_list(&layout, rows),
            Focus::Reader => {
                let action = if rows < 0 { Action::Up } else { Action::Down };
                for _ in 0..usize::from(rows.unsigned_abs()) * WHEEL_ROWS {
                    self.scroll_reader(action, &layout);
                }
            }
        }
        true
    }

    pub(crate) fn hit(&self, layout: &Layout, col: u16, row: u16) -> Hit {
        let rows = visible_rows(&self.mail.folders);
        let spacing = layout.sidebar.as_ref().map(|sidebar| sidebar.spacing);
        let offsets = spacing.map(|spacing| row_offsets(&rows, spacing)).unwrap_or_default();
        let height = folders_height(layout);
        let view = HitView {
            layout,
            folder_offsets: &offsets,
            folders_first: self.folders_view_first(&rows, &offsets, height),
            list_first: self.list_first(layout),
            messages: self.shown_page().map_or(0, |page| page.envelopes.len()),
            accounts: self.mail.accounts.len(),
            key_bar_actions: &self.config.key_bar.buttons,
        };
        hit_at(&view, col, row)
    }

    /// The page of the selected folder, or `None` while another folder's page
    /// is still on screen.
    pub(crate) fn shown_page(&self) -> Option<&ListPage> {
        let folder = self.selection.folder?;
        self.mail.page.as_ref().filter(|page| page.folder == folder)
    }

    pub(crate) fn list_first(&self, layout: &Layout) -> usize {
        let visible = layout.list.as_ref().map_or(0, |list| usize::from(list.rows.height));
        let total = self.shown_page().map_or(0, |page| page.envelopes.len());
        self.list_view_first(visible, total)
    }
}

/// Into the next pane: from the folders to the list, from the list to the
/// message.
fn deeper(focus: Focus) -> Focus {
    match focus {
        Focus::Sidebar => Focus::List,
        Focus::List | Focus::Reader => Focus::Reader,
    }
}

/// Back out to the previous pane.
fn shallower(focus: Focus) -> Focus {
    match focus {
        Focus::Reader => Focus::List,
        Focus::List | Focus::Sidebar => Focus::Sidebar,
    }
}

#[cfg(test)]
include!("dispatch.tests.rs");
