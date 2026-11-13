//! The screen's state and its drawing: events from the backend go in, a
//! frame comes out. No mouse and no key handling yet.

use model::config::Config;
use model::{ConnectionState, Event, MailDate};
use surface::{Frame, Rect};

use crate::app_identity::AppIdentity;
use crate::format::display_width;
use crate::icons::{IconId, IconSet};
use crate::layout::{
    FolderSpacing, GridSize, Layout, LayoutOptions, LayoutPrefs, MIN_COLS, MIN_ROWS, SectionKind,
    SidebarContent, SidebarLayout, compute_layout,
};
use crate::theme::{Theme, roles};
use crate::view_state::{
    Focus, MailSnapshot, Overlays, ScrollState, Selection, first_row_at_offsets,
};
use crate::widgets::{
    DrawContext, FolderTreeView, FooterData, MailboxSummary, StatusLine, StripData,
    content_rows, draw_account_list, draw_account_strip, draw_banner, draw_chrome_lines,
    draw_folder_tree, draw_footer, draw_key_bar, draw_section_title, draw_status_panel,
    fill_cells, put_text, row_offsets, visible_rows,
};

pub struct AppView {
    identity: AppIdentity,
    theme: Theme,
    config: Config,
    icons: IconSet,
    now: MailDate,
    prefs: LayoutPrefs,
    mail: MailSnapshot,
    selection: Selection,
    scroll: ScrollState,
    focus: Focus,
    overlays: Overlays,
}

impl AppView {
    pub fn new(identity: AppIdentity, theme: Theme, config: Config, icons: IconSet) -> Self {
        Self {
            identity,
            theme,
            config,
            icons,
            now: MailDate::default(),
            prefs: LayoutPrefs::default(),
            mail: MailSnapshot::default(),
            selection: Selection::default(),
            scroll: ScrollState::default(),
            focus: Focus::default(),
            overlays: Overlays::default(),
        }
    }

    /// The moment the screen treats as the present, for Today and Yesterday
    /// and for clock times.
    pub fn set_now(&mut self, now: MailDate) {
        self.now = now;
    }

    /// Replaces the preferred sizes (a drag or a restored state file).
    pub fn set_prefs(&mut self, prefs: LayoutPrefs) {
        self.prefs = prefs;
    }

    /// Stores what one backend event carries for the regions.
    pub fn apply_event(&mut self, event: Event) {
        self.mail.apply(event, &mut self.selection);
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn mail(&self) -> &MailSnapshot {
        &self.mail
    }

    pub fn focus(&self) -> Focus {
        self.focus
    }

    pub fn overlays(&self) -> &Overlays {
        &self.overlays
    }

    pub fn identity(&self) -> &AppIdentity {
        &self.identity
    }

    /// The layout of a `cols` x `rows` grid for the data held now.
    pub fn layout(&self, cols: u16, rows: u16) -> Layout {
        let folders = visible_rows(&self.mail.folders);
        let count = |len: usize| u16::try_from(len).unwrap_or(u16::MAX);
        let content = SidebarContent {
            folder_rows_roomy: content_rows(&folders, FolderSpacing::Roomy),
            folder_rows_compact: content_rows(&folders, FolderSpacing::Compact),
            account_rows: count(self.mail.accounts.len()),
            status_lines: count(StatusLine::list_from_names(&self.config.sidebar.status_lines).len()),
        };
        let options = LayoutOptions::from_config(&self.config, content);
        compute_layout(&self.prefs, &options, GridSize::new(cols, rows))
    }

    /// Draws the whole screen for a `cols` x `rows` grid.
    pub fn render(&self, cols: u16, rows: u16) -> Frame {
        let mut frame = Frame::new(Rect::new(0, 0, cols, rows));
        let ctx = DrawContext::new(&self.theme, &self.icons, &self.config, self.now);
        let area = frame.area();
        fill_cells(&mut frame, area, ctx.full(roles::BASE));
        let layout = self.layout(cols, rows);
        if layout.too_small {
            draw_too_small(&mut frame, &ctx);
            return frame;
        }
        // The list and the reader are filled in by a later batch; for now
        // they are just the screen background, under the lines drawn next.
        for rect in layout.list.iter().map(|l| l.rect).chain(layout.reader.iter().map(|r| r.rect)) {
            fill_cells(&mut frame, rect, ctx.full(roles::BASE));
        }
        draw_chrome_lines(&mut frame, &ctx, &layout);
        self.draw_bands(&mut frame, &ctx, &layout);
        if let Some(sidebar) = &layout.sidebar {
            self.draw_sidebar(&mut frame, &ctx, sidebar);
        }
        frame
    }

    fn mailbox_summary(&self) -> Option<MailboxSummary> {
        let folder = self.mail.folder(self.selection.folder?)?;
        Some(MailboxSummary {
            name: folder.name.clone(),
            unread: folder.unread,
            total: folder.total,
        })
    }

    fn connection(&self) -> ConnectionState {
        self.selected_account().map_or(ConnectionState::Offline, |account| account.state)
    }

    fn selected_account(&self) -> Option<&model::Account> {
        self.selection.account.as_ref().and_then(|id| self.mail.account(id))
    }

    fn draw_bands(&self, frame: &mut Frame, ctx: &DrawContext<'_>, layout: &Layout) {
        let mailbox = self.mailbox_summary();
        if let Some(rect) = layout.banner {
            draw_banner(frame, ctx, rect, &self.identity);
        }
        if let Some(rect) = layout.strip {
            let data = StripData {
                account: self.selected_account(),
                mailbox: mailbox.as_ref(),
                host: self.mail.host.as_ref(),
                connection: self.connection(),
            };
            draw_account_strip(frame, ctx, rect, &data);
        }
        if let Some(rect) = layout.key_bar {
            draw_key_bar(frame, ctx, rect, &self.config.key_bar.buttons);
        }
        if let Some(rect) = layout.footer {
            let data = FooterData {
                identity: &self.identity,
                connection: self.connection(),
                mailbox: mailbox.as_ref(),
                status: &self.mail.status,
            };
            draw_footer(frame, ctx, rect, &data);
        }
    }

    fn draw_sidebar(&self, frame: &mut Frame, ctx: &DrawContext<'_>, sidebar: &SidebarLayout) {
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
        let first = match rows.iter().position(|row| Some(row.folder.id) == selected) {
            Some(index) => first_row_at_offsets(&offsets, self.scroll.folders, index, body.height),
            None => self.scroll.folders.min(rows.len().saturating_sub(1)),
        };
        let view = FolderTreeView {
            rows: &rows,
            selected,
            spacing,
            first,
        };
        draw_folder_tree(frame, ctx, body, &view);
    }
}

/// The one line shown when the grid is under the minimum size.
fn draw_too_small(frame: &mut Frame, ctx: &DrawContext<'_>) {
    let area = frame.area();
    let text = format!("Window too small (needs {MIN_COLS} x {MIN_ROWS})");
    let short = "Window too small";
    let text = if display_width(&text) <= usize::from(area.width) { text } else { short.to_string() };
    let width = u16::try_from(display_width(&text)).unwrap_or(u16::MAX);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height / 2;
    put_text(frame, area, x, y, &text, ctx.fg(roles::FOOTER_ERROR));
}

#[cfg(test)]
include!("app_view.tests.rs");
