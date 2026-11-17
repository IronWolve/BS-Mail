//! The sidebar's drawing: its sections, and the folder tree inside them.

use surface::{Frame, Rect};

use crate::app_view::AppView;
use crate::icons::IconId;
use crate::layout::{FolderSpacing, SectionKind, SidebarLayout};
use crate::theme::roles;
use crate::view_focus::FocusLook;
use crate::view_state::Focus;
use crate::widgets::{
    DrawContext, FolderTreeView, StatusLine, draw_account_list, draw_folder_tree, draw_scrollbar, draw_section_title,
    draw_status_panel, row_offsets, visible_rows,
};

impl AppView {
    pub(crate) fn draw_sidebar(&self, frame: &mut Frame, ctx: &DrawContext<'_>, sidebar: &SidebarLayout, look: FocusLook) {
        // The keys are in the folders unless the cursor has run on into the accounts.
        let in_sidebar = look.pane == Focus::Sidebar;
        let cursor = self.account_cursor_row();
        let bar = |focused: bool| look.bar(ctx, focused, roles::SIDEBAR_SELECTED, roles::SIDEBAR_SELECTED_UNFOCUSED);
        for section in &sidebar.sections {
            let (icon, title) = match section.kind {
                SectionKind::Folders => (IconId::FolderOpen, "Folders"),
                SectionKind::Accounts => (IconId::At, "Accounts"),
                SectionKind::Status => (IconId::Chart, "Status"),
            };
            draw_section_title(frame, ctx, section.title, icon, title);
            match section.kind {
                SectionKind::Folders => {
                    self.draw_folders(frame, ctx, section.body, sidebar.spacing, bar(in_sidebar && cursor.is_none()));
                    if let Some(bar) = self.folders_bar(sidebar) {
                        draw_scrollbar(frame, ctx, bar.col, bar.rows, bar.first, bar.visible, bar.total);
                    }
                }
                SectionKind::Accounts => draw_account_list(
                    frame,
                    ctx,
                    section.body,
                    &self.mail.accounts,
                    self.selection.account.as_ref(),
                    cursor,
                    bar(in_sidebar),
                ),
                SectionKind::Status => {
                    let lines = StatusLine::list_from_names(&self.config.sidebar.status_lines);
                    draw_status_panel(frame, ctx, section.body, &self.mail.status, &lines);
                }
            }
        }
    }

    fn draw_folders(
        &self,
        frame: &mut Frame,
        ctx: &DrawContext<'_>,
        body: Rect,
        spacing: FolderSpacing,
        bar: &'static str,
    ) {
        let rows = visible_rows(&self.mail.folders);
        let offsets = row_offsets(&rows, spacing);
        let selected = self.selection.folder;
        let first = self.folders_view_first(&rows, &offsets, body.height);
        let view = FolderTreeView {
            rows: &rows,
            selected,
            bar,
            spacing,
            first,
        };
        draw_folder_tree(frame, ctx, body, &view);
    }
}
