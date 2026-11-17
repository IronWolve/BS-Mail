//! Drawing the bands of the screen around the panes: the banner, the account
//! strip, the key bar and the footer.

use surface::Frame;

use crate::app_view::AppView;
use crate::format::display_width;
use crate::input::link_light::LitLink;
use crate::layout::{Layout, MIN_COLS, MIN_ROWS, SectionKind};
use crate::theme::roles;
use crate::view_state::Focus;
use crate::widgets::{
    DrawContext, FooterData, MailboxSummary, StripData, draw_account_strip, draw_banner, draw_footer_with_hint, draw_key_bar,
    mark_focused, put_text,
};

impl AppView {
    pub(crate) fn draw_bands(&self, frame: &mut Frame, ctx: &DrawContext<'_>, layout: &Layout, lit: Option<&LitLink>) {
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
            let hint = self.link_hint(lit);
            draw_footer_with_hint(frame, ctx, rect, &self.footer_data(mailbox.as_ref()), hint.as_ref());
        }
    }

    /// What the footer shows now.
    pub(crate) fn footer_data<'a>(&'a self, mailbox: Option<&'a MailboxSummary>) -> FooterData<'a> {
        FooterData {
            identity: &self.identity,
            connection: self.connection(),
            mailbox,
            status: &self.mail.status,
            error: self.footer_note(),
            notice: self.new_mail_notice(),
        }
    }

    /// Underlines the title of the pane that has focus.
    pub(crate) fn mark_focus(&self, frame: &mut Frame, layout: &Layout) {
        let title = match self.focus {
            Focus::Sidebar => {
                let kind = if self.account_cursor_row().is_some() { SectionKind::Accounts } else { SectionKind::Folders };
                layout.sidebar.as_ref().and_then(|sidebar| sidebar.section(kind)).map(|section| section.title)
            }
            Focus::List => layout.list.as_ref().map(|list| list.title),
            Focus::Reader => layout.reader.as_ref().map(|reader| reader.title),
        };
        if let Some(title) = title {
            mark_focused(frame, title);
        }
    }
}

/// The one line shown when the grid is under the minimum size.
pub(crate) fn draw_too_small(frame: &mut Frame, ctx: &DrawContext<'_>) {
    let area = frame.area();
    let text = format!("Window too small (needs {MIN_COLS} x {MIN_ROWS})");
    let short = "Window too small";
    let text = if display_width(&text) <= usize::from(area.width) { text } else { short.to_string() };
    let width = u16::try_from(display_width(&text)).unwrap_or(u16::MAX);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height / 2;
    put_text(frame, area, x, y, &text, ctx.fg(roles::FOOTER_ERROR));
}
