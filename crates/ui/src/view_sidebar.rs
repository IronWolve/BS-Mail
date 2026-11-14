//! The sidebar's drawing: its sections, and the folder tree inside them.

use surface::{Frame, Rect};

use crate::app_view::AppView;
use crate::icons::IconId;
use crate::input::viewport::folders_first;
use crate::layout::{FolderSpacing, SectionKind, SidebarLayout};
use crate::widgets::{
    DrawContext, FolderTreeView, StatusLine, draw_account_list, draw_folder_tree, draw_section_title,
    draw_status_panel, row_offsets, visible_rows,
};

impl AppView {
    pub(crate) fn draw_sidebar(&self, frame: &mut Frame, ctx: &DrawContext<'_>, sidebar: &SidebarLayout) {
        for section in &sidebar.sections {
            let (icon, title) = match section.kind {
                SectionKind::Folders => (IconId::FolderOpen, "Folders"),
                SectionKind::Accounts => (IconId::At, "Accounts"),
                SectionKind::Status => (IconId::Chart, "Status"),
            };
            draw_section_title(frame, ctx, section.title, icon, title);
            match section.kind {
                SectionKind::Folders => self.draw_folders(frame, ctx, section.body, sidebar.spacing),
                SectionKind::Accounts => draw_account_list(
                    frame,
                    ctx,
                    section.body,
                    &self.mail.accounts,
                    self.selection.account.as_ref(),
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
    ) {
        let rows = visible_rows(&self.mail.folders);
        let offsets = row_offsets(&rows, spacing);
        let selected = self.selection.folder;
        let first = folders_first(&rows, &offsets, selected, self.scroll.folders, body.height);
        let view = FolderTreeView {
            rows: &rows,
            selected,
            spacing,
            first,
        };
        draw_folder_tree(frame, ctx, body, &view);
    }
}
