//! Facts the view works out about itself from its state, which the drawing
//! and the host both ask for: the window's title, the open mailbox, the
//! connection of the selected account.

use std::borrow::Cow;

use model::{BodyRole, ConnectionState, StyledLine, StyledSpan};

use crate::app_view::AppView;
use crate::format::format_size;
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
    /// results are showing, with how many of them are unread. While messages
    /// are marked it also says how many, since the keys that change mail act
    /// on them and the marks themselves may not fit on a narrow list.
    pub(crate) fn list_summary(&self) -> Option<MailboxSummary> {
        let mut summary = self.list_summary_without_marks()?;
        if !self.marked.is_empty() {
            summary.name = format!("{}, {} marked", summary.name, self.marked.len());
        }
        Some(summary)
    }

    fn list_summary_without_marks(&self) -> Option<MailboxSummary> {
        let Some(search) = &self.search else {
            return self.mailbox_summary();
        };
        let page = self.mail.page.as_ref()?;
        let unread = page.envelopes.iter().filter(|envelope| !envelope.flags.seen).count();
        let marker = if search.incomplete.is_some() { " (incomplete)" } else { "" };
        Some(MailboxSummary {
            name: format!("Search: {}{marker}", search.query),
            unread: u32::try_from(unread).unwrap_or(u32::MAX),
            total: page.total,
        })
    }

    /// Tells the view what was wrong with the user's settings, if anything.
    pub fn set_config_problem(&mut self, problem: Option<String>) {
        self.config_problem = problem;
    }

    /// The lines the reader shows for the selected message: a line for each
    /// attachment, then a blank, then the body. `None` while its body is not
    /// here yet. Borrowed whole when there are no attachments.
    pub(crate) fn reader_lines(&self) -> Option<Cow<'_, [StyledLine]>> {
        let (_, body) = self.mail.body.as_ref().filter(|(id, _)| Some(*id) == self.selection.message)?;
        if body.attachments.is_empty() {
            return Some(Cow::Borrowed(body.lines.as_slice()));
        }
        let mut lines: Vec<StyledLine> = body
            .attachments
            .iter()
            .map(|attachment| StyledLine {
                spans: vec![StyledSpan {
                    text: format!(
                        "Attachment: {} ({}, {})",
                        attachment.name,
                        attachment.content_type,
                        format_size(attachment.size, self.config.list.size_format)
                    ),
                    role: BodyRole::Path,
                }],
            })
            .collect();
        lines.push(StyledLine::default());
        lines.extend(body.lines.iter().cloned());
        Some(Cow::Owned(lines))
    }
}
