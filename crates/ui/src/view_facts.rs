//! Facts the view works out about itself from its state, which the drawing
//! and the host both ask for: the window's title, the open mailbox, the
//! connection of the selected account.

use model::ConnectionState;

use crate::app_view::AppView;
use crate::widgets::MailboxSummary;

impl AppView {
    /// The window's title: the name, then the open folder and its unread
    /// count when the options ask for them.
    pub fn window_title(&self) -> String {
        let name = self.identity.name.clone();
        if !self.config.window.title_shows_folder {
            return name;
        }
        let Some(folder) = self.selection.folder.and_then(|id| self.mail.folder(id)) else {
            return name;
        };
        match folder.unread {
            0 => format!("{name} \u{2014} {}", folder.name),
            unread => format!("{name} \u{2014} {} ({unread})", folder.name),
        }
    }

    pub(crate) fn mailbox_summary(&self) -> Option<MailboxSummary> {
        let folder = self.mail.folder(self.selection.folder?)?;
        Some(MailboxSummary {
            name: folder.name.clone(),
            unread: folder.unread,
            total: folder.total,
        })
    }

    pub(crate) fn connection(&self) -> ConnectionState {
        self.selected_account().map_or(ConnectionState::Offline, |account| account.state)
    }

    pub(crate) fn selected_account(&self) -> Option<&model::Account> {
        self.selection.account.as_ref().and_then(|id| self.mail.account(id))
    }

    /// What the list's title names: the open mailbox, or the search whose
    /// results are showing, with how many of them are unread.
    pub(crate) fn list_summary(&self) -> Option<MailboxSummary> {
        let Some(search) = &self.search else {
            return self.mailbox_summary();
        };
        let page = self.mail.page.as_ref()?;
        let unread = page.envelopes.iter().filter(|envelope| !envelope.flags.seen).count();
        Some(MailboxSummary {
            name: format!("Search: {}", search.query),
            unread: u32::try_from(unread).unwrap_or(u32::MAX),
            total: page.total,
        })
    }
}
