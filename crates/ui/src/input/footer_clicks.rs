//! The footer items that do something when clicked: the connection syncs now,
//! "Press ? for help" opens the help, and the notice of new mail goes to that
//! account (to its Inbox when it is the one on screen).

use surface::Rect;

use crate::app_view::AppView;
use crate::layout::Layout;
use crate::view_state::OverlayKind;
use crate::widgets::{DrawContext, FooterItem, FooterSpan, footer_item_at};

impl AppView {
    /// The clickable footer segment at cell (`col`, `row`), if there is one.
    pub(crate) fn footer_span_at(&self, layout: &Layout, col: u16, row: u16) -> Option<FooterSpan> {
        let rect: Rect = layout.footer.filter(|rect| row == rect.y)?;
        let ctx = DrawContext::new(&self.theme, &self.icons, &self.config, self.now);
        let mailbox = self.mailbox_summary();
        footer_item_at(&ctx, rect, &self.footer_data(mailbox.as_ref()), col)
    }

    /// A click on the footer. Returns whether it was on something that acts.
    pub(crate) fn click_footer(&mut self, span: FooterSpan) -> bool {
        match span.item {
            FooterItem::Connection => self.refresh(),
            FooterItem::Help => {
                self.overlays.open = Some(OverlayKind::Help);
                self.overlays.help_scroll = 0;
            }
            FooterItem::NewMail => self.go_to_new_mail(),
        }
        true
    }

    /// Shows the account the notice is about, and its Inbox when it is the
    /// one on screen; the notice goes.
    fn go_to_new_mail(&mut self) {
        let Some(account) = self.new_mail_account() else {
            return;
        };
        self.new_mail = None;
        if self.selection.account.as_ref() != Some(&account) {
            self.switch_account(account);
            return;
        }
        let inbox = self.mail.folders.iter().find(|folder| folder.role == model::FolderRole::Inbox).map(|folder| folder.id);
        if let Some(inbox) = inbox {
            self.account_cursor = None;
            self.change_folder(inbox);
        }
    }
}

#[cfg(test)]
include!("footer_clicks.tests.rs");
