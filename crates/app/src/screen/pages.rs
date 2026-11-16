//! The pages of HTML messages for the browser: the screen asks for a safe copy
//! of a message (`b`), the engine makes it, and here it is written to a private
//! file in `html/` in the data folder and the file is handed to the opener,
//! which opens only files of that folder. A page that cannot be written says
//! why in the footer; the footer says how many remote items were left out once
//! the browser has the file.

use std::io;
use std::path::Path;

use super::Screen;
use crate::html_store::HtmlStore;

impl Screen {
    /// Writes the pages of HTML messages to files in `html/` in the data folder
    /// `data`, which is where the opener must be told to open from too.
    pub fn keep_pages_in(mut self, data: &Path) -> Self {
        self.pages = Some(HtmlStore::open(data));
        self
    }

    /// Writes the pages the view has made and hands each file to the opener.
    /// Returns whether there was one.
    pub(super) fn page_steps(&mut self) -> bool {
        let pages = self.view.take_html_pages();
        let any = !pages.is_empty();
        for page in pages {
            let written = match self.pages.as_mut() {
                Some(store) => store.write(&page.html),
                None => Err(io::Error::other("this run keeps no data folder")),
            };
            match written {
                Ok(path) => {
                    self.page_removed = Some(page.removed);
                    self.links.open_file(&path);
                }
                Err(error) => self.view.link_open_failed(&format!("the page could not be saved for the browser: {error}")),
            }
        }
        any
    }
}

#[cfg(test)]
include!("pages.tests.rs");
