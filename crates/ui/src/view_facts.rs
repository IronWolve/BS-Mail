//! Facts the view works out about itself from its state, which the drawing
//! and the host both ask for: the window's title, the open mailbox, the
//! connection of the selected account.

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
        // The title names the search only once its answer is what the rows
        // are; until then they are still the folder's own.
        let Some(query) = self.search.as_ref().and_then(|search| search.showing.as_ref()) else {
            return self.mailbox_summary();
        };
        let page = self.mail.page.as_ref()?;
        let unread = page.envelopes.iter().filter(|envelope| !envelope.flags.seen).count();
        let incomplete = self.search.as_ref().is_some_and(|search| search.incomplete.is_some());
        let marker = if incomplete { " (incomplete)" } else { "" };
        Some(MailboxSummary {
            name: format!("Search: {query}{marker}"),
            unread: u32::try_from(unread).unwrap_or(u32::MAX),
            total: page.total,
        })
    }

    /// Tells the view what was wrong with the user's settings, if anything.
    pub fn set_config_problem(&mut self, problem: Option<String>) {
        self.config_problem = problem;
    }

    /// What the reader shows for the selected message: a line for each
    /// attachment and a blank (`lead`), then the lines of the body. `None`
    /// while its body is not here yet. The body is borrowed, never copied, so
    /// a long message costs nothing more to scroll for having an attachment.
    pub(crate) fn reader_content(&self) -> Option<ReaderContent<'_>> {
        // The raw headers or source, while they are what the pane was asked to show.
        if let Some(raw) = self.reading.raw_of(self.selection.message) {
            return Some(ReaderContent { lead: vec![raw.heading(), StyledLine::default()], body: raw.lines() });
        }
        let (_, body) = self.mail.body.as_ref().filter(|(id, _)| Some(*id) == self.selection.message)?;
        // While the cursor is on one attachment, each line has a two cell
        // gutter and the chosen one a `>` in it and the lead colour.
        let cursor = self.attachment_cursor();
        let mut lead: Vec<StyledLine> = body
            .attachments
            .iter()
            .enumerate()
            .map(|(at, attachment)| {
                let marker = match cursor {
                    None => "",
                    Some(chosen) if chosen == at => "> ",
                    Some(_) => "  ",
                };
                StyledLine {
                    spans: vec![StyledSpan {
                        text: format!(
                            "{marker}Attachment: {} ({}, {})",
                            attachment.name,
                            attachment.content_type,
                            format_size(attachment.size, self.config.list.size_format)
                        ),
                        role: if cursor == Some(at) { BodyRole::LeadNext } else { BodyRole::Path },
                    }],
                }
            })
            .collect();
        if !lead.is_empty() {
            lead.push(StyledLine::default());
        }
        Some(ReaderContent { lead, body: body.lines.as_slice() })
    }
}

/// The lines the reader draws, in two parts so the body is never copied.
pub(crate) struct ReaderContent<'a> {
    /// The attachment lines and the blank after them; empty with none.
    pub lead: Vec<StyledLine>,
    pub body: &'a [StyledLine],
}

#[cfg(test)]
include!("view_facts.tests.rs");
