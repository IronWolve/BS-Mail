//! Drawing the bands of the screen around the panes: the banner, the account
//! strip, the key bar and the footer.

use surface::Frame;

use crate::app_view::AppView;
use crate::layout::Layout;
use crate::widgets::{DrawContext, FooterData, StripData, draw_account_strip, draw_banner, draw_footer, draw_key_bar};

impl AppView {
    pub(crate) fn draw_bands(&self, frame: &mut Frame, ctx: &DrawContext<'_>, layout: &Layout) {
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
                notice: self.new_mail_notice(),
            };
            draw_footer(frame, ctx, rect, &data);
        }
    }
}
