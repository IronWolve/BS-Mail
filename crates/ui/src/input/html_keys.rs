//! Opening a message in the browser: `b` asks the engine for a safe copy of
//! the message's HTML (scripts, frames, forms, styles that name files and
//! everything that would load from the network are left out and counted), and
//! when it comes the program writes it to a private file and hands the file to
//! the browser. The screen only asks and tells: the footer says what is
//! happening and, at the end, how many remote items were left out.

use model::{Command, MessageId};

use crate::app_view::AppView;
use crate::reader_tools::HtmlPage;

impl AppView {
    /// `b`: asks for the safe copy of the selected message's HTML.
    pub(crate) fn open_in_browser(&mut self) {
        let Some(message) = self.message_to_read() else {
            return;
        };
        if self.reading.page_asked == Some(message) {
            return;
        }
        self.reading.page_asked = Some(message);
        self.mail.set_note(Some("Preparing a safe copy of the message for your browser...".to_owned()));
        self.queue.push(Command::PrepareHtml { message });
    }

    /// The safe copy asked for arrived: it waits for the program to take it.
    pub(crate) fn page_arrived(&mut self, message: MessageId, html: &str, removed: u32) {
        if self.reading.page_asked != Some(message) {
            return;
        }
        self.reading.page_asked = None;
        self.reading.pages.push(HtmlPage { html: html.to_owned(), removed });
    }

    /// The pages made and not yet taken, once, for the program to write to
    /// files and open.
    pub fn take_html_pages(&mut self) -> Vec<HtmlPage> {
        std::mem::take(&mut self.reading.pages)
    }

    /// The browser took the page: the footer says how much was left out of it.
    pub fn html_opened(&mut self, removed: u32) {
        let left_out = match removed {
            0 => "no remote items removed".to_owned(),
            1 => "1 remote item removed".to_owned(),
            n => format!("{n} remote items removed"),
        };
        self.mail.set_note(Some(format!("Opened in your browser ({left_out})")));
    }
}

#[cfg(test)]
include!("html_keys.tests.rs");
