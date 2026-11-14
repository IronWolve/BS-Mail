//! The screen's state and its drawing: events from the backend go in, a
//! frame comes out. No mouse and no key handling yet.

use std::collections::BTreeSet;

use model::config::Config;
use model::{Event, MailDate, MessageId};
use surface::{Capabilities, Frame, Rect};

use crate::app_identity::AppIdentity;
use crate::format::display_width;
use crate::icons::IconSet;
use crate::composer::ComposeForm;
use crate::composer::EditorBuffer;
use crate::move_picker::MovePicker;
use crate::options::OptionsState;
use crate::options::draw::draw_options;
use crate::options::empty_hint::draw_no_account_hint;
use crate::options::host::OptionsHost;
use crate::search::ActiveSearch;
use crate::setup::SetupForm;
use crate::setup::draw::draw_setup;
use crate::search::draw::draw_search_bar;
use crate::move_picker::draw::draw_picker;
use crate::composer::draw::draw_composer;
use crate::input::{CommandQueue, Drag};
use crate::input::viewport::list_first;
use crate::layout::{
    FolderSpacing, GridSize, Layout, LayoutOptions, LayoutPrefs, ListRegion, MIN_COLS, MIN_ROWS,
    ReaderRegion, SectionKind, SidebarContent, compute_layout,
};
use crate::theme::{Theme, roles};
use crate::view_state::{
    Focus, ListPage, MailSnapshot, OverlayKind, Overlays, ScrollState, Selection,
};
use crate::widgets::{
    BodyView, DrawContext, FooterData, HighlightRules, ListRowsView,
    ReaderTitleData, StatusLine, StripData, content_rows,
    draw_account_strip, draw_banner, draw_chrome_lines, draw_footer,
    draw_help, draw_key_bar, mark_focused, draw_list_header, draw_list_rows, draw_list_title, draw_reader_body,
    draw_reader_headers, draw_reader_title, fill_cells,
    put_text, visible_rows,
};

pub struct AppView {
    pub(crate) identity: AppIdentity,
    pub(crate) theme: Theme,
    pub(crate) config: Config,
    pub(crate) icons: IconSet,
    capabilities: Capabilities,
    pub(crate) now: MailDate,
    pub(crate) prefs: LayoutPrefs,
    pub(crate) mail: MailSnapshot,
    pub(crate) selection: Selection,
    pub(crate) scroll: ScrollState,
    pub(crate) focus: Focus,
    pub(crate) overlays: Overlays,
    pub(crate) queue: CommandQueue,
    /// The message that will be marked read, and when (milliseconds on the
    /// caller's clock).
    pub(crate) read_timer: Option<(MessageId, u64)>,
    /// The message this view already marked read by itself, so it is not
    /// marked again right after the user marks it unread.
    pub(crate) auto_marked: Option<MessageId>,
    /// The bar being dragged, while a button is held on it.
    pub(crate) drag: Option<Drag>,
    /// The message being written, when the composer is open.
    pub(crate) composer: Option<ComposeForm>,
    /// The user clicked Quit; the program closes it, the screen cannot.
    pub(crate) quit_requested: bool,
    /// Messages marked with Space, which the change keys then act on together.
    pub(crate) marked: BTreeSet<MessageId>,
    /// The folder picker, while one is open to move messages.
    pub(crate) picker: Option<MovePicker>,
    /// The search box, while one is open to take a query.
    pub(crate) search_bar: Option<EditorBuffer>,
    /// The search whose results are showing in the list.
    pub(crate) search: Option<ActiveSearch>,
    /// The number of the last search asked for, so answers can be told apart.
    pub(crate) next_request: u64,
    /// What was wrong with the user's settings, shown in the footer for as
    /// long as nothing more urgent is.
    pub(crate) config_problem: Option<String>,
    /// The account form, while the program has no account yet.
    pub(crate) setup: Option<SetupForm>,
    /// The account file text the user asked to try, until the program takes it.
    pub(crate) setup_request: Option<String>,
    /// The account that was tried has signed in; the program saves it next.
    pub(crate) setup_online: bool,
    /// What the form asked the program to do, not yet taken.
    pub(crate) setup_actions: Vec<crate::setup::SetupAction>,
    /// The Options screen, while it is open.
    pub(crate) options: Option<OptionsState>,
    /// What the program told the Options screen, and what it asked for.
    pub(crate) options_host: OptionsHost,
}

impl AppView {
    pub fn new(identity: AppIdentity, theme: Theme, config: Config, icons: IconSet) -> Self {
        Self {
            identity,
            theme,
            config,
            icons,
            capabilities: Capabilities::window(),
            now: MailDate::default(),
            prefs: LayoutPrefs::default(),
            mail: MailSnapshot::default(),
            selection: Selection::default(),
            scroll: ScrollState::default(),
            focus: Focus::default(),
            overlays: Overlays::default(),
            queue: CommandQueue::default(),
            read_timer: None,
            auto_marked: None,
            drag: None,
            composer: None,
            quit_requested: false,
            marked: BTreeSet::new(),
            picker: None,
            search_bar: None,
            search: None,
            next_request: 0,
            config_problem: None,
            setup: None,
            setup_request: None,
            setup_online: false,
            setup_actions: Vec::new(),
            options: None,
            options_host: OptionsHost::default(),
        }
    }

    /// The moment the screen treats as the present, for Today and Yesterday
    /// and for clock times.
    pub fn set_now(&mut self, now: MailDate) {
        self.now = now;
    }

    /// What the host showing these frames can do. A window is assumed until
    /// a host says otherwise, so the screen is laid out for every effect.
    pub fn set_capabilities(&mut self, capabilities: Capabilities) {
        self.capabilities = capabilities;
    }

    pub fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    /// The preferred sizes now, for saving after a drag.
    pub fn prefs(&self) -> &LayoutPrefs {
        &self.prefs
    }

    /// Replaces the preferred sizes (a drag or a restored state file).
    pub fn set_prefs(&mut self, prefs: LayoutPrefs) {
        self.prefs = prefs;
    }

    /// Stores what one backend event carries for the regions.
    pub fn apply_event(&mut self, event: Event) {
        self.note_setup_event(&event);
        self.note_send_outcome(&event);
        self.note_search_event(&event);
        self.mail.apply(event, &mut self.selection);
        self.prune_marks();
    }

    /// How many messages are marked.
    pub fn marked_count(&self) -> usize {
        self.marked.len()
    }

    /// Whether the user asked to quit from the screen, by the Quit button.
    pub fn wants_quit(&self) -> bool {
        self.quit_requested
    }

    /// Whether the composer is open, so the program leaves the letters to it.
    pub fn is_composing(&self) -> bool {
        self.composer.is_some()
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
        let options = LayoutOptions::from_config(&self.config, content)
            .fit_to_host(&self.capabilities.effects);
        compute_layout(&self.prefs, &options, GridSize::new(cols, rows))
    }

    /// Draws the whole screen for a `cols` x `rows` grid.
    pub fn render(&self, cols: u16, rows: u16) -> Frame {
        let mut frame = Frame::new(Rect::new(0, 0, cols, rows));
        let ctx = DrawContext::new(&self.theme, &self.icons, &self.config, self.now);
        let area = frame.area();
        if let Some(form) = &self.setup {
            draw_setup(&mut frame, &ctx, area, form);
            return frame;
        }
        if let Some(state) = &self.options {
            draw_options(&mut frame, &ctx, area, &self.options_view(), state);
            return frame;
        }
        fill_cells(&mut frame, area, ctx.full(roles::BASE));
        let layout = self.layout(cols, rows);
        if layout.too_small {
            draw_too_small(&mut frame, &ctx);
            return frame;
        }
        draw_chrome_lines(&mut frame, &ctx, &layout);
        self.draw_bands(&mut frame, &ctx, &layout);
        if let Some(sidebar) = &layout.sidebar {
            self.draw_sidebar(&mut frame, &ctx, sidebar);
        }
        if let Some(list) = &layout.list {
            self.draw_list(&mut frame, &ctx, list);
        }
        if let Some(reader) = &layout.reader {
            self.draw_reader(&mut frame, &ctx, reader);
        }
        self.mark_focus(&mut frame, &layout);
        if let Some(form) = &self.composer {
            draw_composer(&mut frame, &ctx, layout.main, form);
        }
        if let Some(picker) = &self.picker {
            draw_picker(&mut frame, &ctx, area, picker, &self.mail.folders);
        }
        if let Some(bar) = &self.search_bar {
            draw_search_bar(&mut frame, &ctx, area, bar);
        }
        if self.overlays.open == Some(OverlayKind::Help) {
            draw_help(&mut frame, &ctx, area);
        }
        frame
    }

    /// Underlines the title of the pane that has focus.
    fn mark_focus(&self, frame: &mut Frame, layout: &Layout) {
        let title = match self.focus {
            Focus::Sidebar => layout
                .sidebar
                .as_ref()
                .and_then(|sidebar| sidebar.section(SectionKind::Folders))
                .map(|section| section.title),
            Focus::List => layout.list.as_ref().map(|list| list.title),
            Focus::Reader => layout.reader.as_ref().map(|reader| reader.title),
        };
        if let Some(title) = title {
            mark_focused(frame, title);
        }
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
                error: self.mail.error.as_deref().or(self.config_problem.as_deref()),
            };
            draw_footer(frame, ctx, rect, &data);
        }
    }

    /// The envelope of the selected message, from the loaded page.
    fn selected_envelope(&self) -> Option<&model::Envelope> {
        self.mail.page.as_ref()?.envelopes.get(self.selection.message_index)
    }

    fn draw_list(&self, frame: &mut Frame, ctx: &DrawContext<'_>, list: &ListRegion) {
        let mailbox = self.list_summary();
        draw_list_title(frame, ctx, list.title, mailbox.as_ref());
        draw_list_header(frame, ctx, list.header, &list.columns);
        if self.options_host.has_no_account() {
            draw_no_account_hint(frame, ctx, list.rows);
            return;
        }
        let Some(page) = &self.mail.page else {
            return;
        };
        let visible = usize::from(list.rows.height);
        let selected = self.selection.message_index;
        let view = ListRowsView {
            envelopes: &page.envelopes,
            columns: &list.columns,
            selected: Some(selected),
            offset: page.offset,
            first: list_first(self.scroll.list, selected, visible, page.envelopes.len()),
            now: self.now,
        };
        draw_list_rows(frame, ctx, list.rows, &view);
        self.show_marks(frame, list, view.first, &page.envelopes);
    }

    fn reader_title_data<'a>(&'a self, page: &'a ListPage) -> Option<ReaderTitleData<'a>> {
        let envelope = page.envelopes.get(self.selection.message_index)?;
        let unread = self.mailbox_summary().map_or(0, |mailbox| mailbox.unread);
        Some(ReaderTitleData {
            subject: &envelope.subject,
            position: u64::from(page.offset) + self.selection.message_index as u64 + 1,
            total: u64::from(page.total),
            unread,
        })
    }

    fn draw_reader(&self, frame: &mut Frame, ctx: &DrawContext<'_>, reader: &ReaderRegion) {
        let title = self.mail.page.as_ref().and_then(|page| self.reader_title_data(page));
        draw_reader_title(frame, ctx, reader.title, title.as_ref());
        draw_reader_headers(frame, ctx, reader.headers, self.selected_envelope());
        if let Some(lines) = self.reader_lines() {
            let view = BodyView {
                lines: &lines,
                first: self.scroll.reader,
                app_name: &self.identity.name,
                rules: HighlightRules::default(),
            };
            draw_reader_body(frame, ctx, reader.body, &view);
        }
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
