//! Moving the cursors: the folder, the message and the reader's scroll.
//!
//! Each takes a movement (`Up`, `Down`, `PageUp`, `PageDown`, `First`,
//! `Last`) or an index, and changes the selection or the scroll. None of them
//! talks to the backend; the commands the new selection needs are worked out
//! afterwards (see `command_queue`).

use super::action::Action;
use super::movement::step;
use super::viewport::{folders_first, list_first};
use crate::app_view::AppView;
use crate::layout::{Layout, SectionKind};
use crate::widgets::{BodyView, DrawContext, HighlightRules, row_offsets, visible_rows, visual_len};

impl AppView {
    pub(super) fn move_folder(&mut self, action: Action, layout: &Layout) {
        let rows = visible_rows(&self.mail.folders);
        if rows.is_empty() {
            return;
        }
        let height = folders_height(layout);
        let current = rows
            .iter()
            .position(|row| Some(row.folder.id) == self.selection.folder)
            .unwrap_or(0);
        let next = step(action, current, rows.len(), usize::from(height));
        let chosen = rows[next].folder.id;
        let spacing = layout.sidebar.as_ref().map(|sidebar| sidebar.spacing);
        let first = spacing.map(|spacing| {
            let offsets = row_offsets(&rows, spacing);
            folders_first(&rows, &offsets, Some(chosen), self.scroll.folders, height)
        });
        self.change_folder(chosen);
        if let Some(first) = first {
            self.scroll.folders = first;
            self.scroll.folders_detached = None;
        }
    }

    pub(super) fn select_folder_row(&mut self, index: usize) {
        let rows = visible_rows(&self.mail.folders);
        let Some(row) = rows.get(index) else {
            return;
        };
        let chosen = row.folder.id;
        self.change_folder(chosen);
    }

    pub(super) fn move_message(&mut self, action: Action, layout: &Layout) {
        let visible = layout.list.as_ref().map_or(1, |list| usize::from(list.rows.height));
        let Some(page) = self.shown_page() else {
            return;
        };
        let next = step(action, self.selection.message_index, page.envelopes.len(), visible);
        self.select_message(next, layout);
    }

    pub(super) fn select_message(&mut self, index: usize, layout: &Layout) {
        let visible = layout.list.as_ref().map_or(1, |list| usize::from(list.rows.height));
        let Some(page) = self.mail.page.as_ref().filter(|p| Some(p.folder) == self.selection.folder)
        else {
            return;
        };
        let Some(envelope) = page.envelopes.get(index) else {
            return;
        };
        let changed = self.selection.message != Some(envelope.id);
        self.selection.message = Some(envelope.id);
        self.selection.message_index = index;
        self.scroll.list = list_first(self.scroll.list, index, visible, page.envelopes.len());
        self.scroll.list_detached = None;
        if changed {
            self.scroll.reader = 0;
        }
    }

    /// How many wrapped lines the open message has, and how many of them the
    /// reader's body shows at once.
    pub(super) fn reader_extent(&self, layout: &Layout) -> Option<(usize, usize)> {
        let reader = layout.reader.as_ref()?;
        let content = self.reader_content()?;
        let ctx = DrawContext::new(&self.theme, &self.icons, &self.config, self.now);
        let view = BodyView {
            lead: &content.lead,
            lines: content.body,
            first: 0,
            app_name: &self.identity.name,
            rules: HighlightRules::default(),
        };
        Some((visual_len(&ctx, reader.body, &view), usize::from(reader.body.height)))
    }

    pub(super) fn scroll_reader(&mut self, action: Action, layout: &Layout) {
        let Some((lines, height)) = self.reader_extent(layout) else {
            return;
        };
        let last = lines.saturating_sub(height);
        let next = match action {
            Action::Up => self.scroll.reader.saturating_sub(1),
            Action::Down => self.scroll.reader.saturating_add(1),
            Action::PageUp => self.scroll.reader.saturating_sub(height.saturating_sub(1).max(1)),
            Action::PageDown => self.scroll.reader.saturating_add(height.saturating_sub(1).max(1)),
            Action::HalfPageUp => self.scroll.reader.saturating_sub((height / 2).max(1)),
            Action::HalfPageDown => self.scroll.reader.saturating_add((height / 2).max(1)),
            Action::First => 0,
            Action::Last => last,
            _ => self.scroll.reader,
        };
        self.scroll.reader = next.min(last);
    }}

/// How many rows the folder section's body has.
pub(super) fn folders_height(layout: &Layout) -> u16 {
    layout
        .sidebar
        .as_ref()
        .and_then(|sidebar| sidebar.section(SectionKind::Folders))
        .map_or(0, |section| section.body.height)
}
