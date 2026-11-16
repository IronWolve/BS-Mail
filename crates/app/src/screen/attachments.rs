//! The attachments the screen is asked to deal with: a file that arrived from
//! the engine is written to `attachments/` in the data folder (and handed to
//! the system to open for `o`) or to the place the user typed, and the
//! footer says where it went or why not; a file the user named for the message
//! being written is read, with the checks of `attach_read`, and handed back to
//! the composer. All of it is told to the view in plain words.

use std::path::{Path, PathBuf};

use ui::Want;
use ui::composer::attach_line::human_size;

use super::Screen;
use crate::attach_read::{Places, read_for_attach, resolve};
use crate::attachment_name::is_never_opened;
use crate::attachment_store::{AttachmentStore, plain_words, save_into, save_to};

/// What the screen keeps to deal with attachments.
#[derive(Default)]
pub(super) struct Attachments {
    /// Where saved attachments go; `None` for a run that keeps no data folder.
    store: Option<AttachmentStore>,
    /// What reading a typed path needs to know.
    places: Places,
    /// The file the system was asked to open, until it answers.
    opening: Option<Opening>,
}

/// A saved attachment the system is opening: what the footer says of it when
/// the answer comes. What matters comes first, the folder last, because the
/// footer cuts a long line at its end.
pub(super) struct Opening {
    name: String,
    size: String,
    dir: String,
}

impl Opening {
    pub(super) fn opened(&self) -> String {
        format!("Opened {} ({}); saved in {}", self.name, self.size, self.dir)
    }

    pub(super) fn failed(&self, reason: &str) -> String {
        format!("{} ({}) was saved but could not be opened: {reason}. It is in {}", self.name, self.size, self.dir)
    }
}

impl Attachments {
    /// The file waiting for the opener's answer, once.
    pub(super) fn take_opening(&mut self) -> Option<Opening> {
        self.opening.take()
    }
}

impl Screen {
    /// Saves attachments in `attachments/` of the data folder `data`, and
    /// refuses to attach what is in the data folder (except those) or the
    /// account files in `secrets`.
    pub fn keep_attachments_in(mut self, data: &Path, secrets: Vec<PathBuf>) -> Self {
        self.attachments.store = Some(AttachmentStore::open(data));
        self.attachments.places = Places { home: ui::path_entry::home_dir(), data: Some(data.to_path_buf()), secrets };
        self
    }

    /// Writes the attachments that arrived and reads the files the composer
    /// asked for. Returns whether there was anything to do.
    pub(super) fn attachment_steps(&mut self) -> bool {
        let files = self.view.take_attachment_files();
        let asks = self.view.take_attach_asks();
        let any = !files.is_empty() || !asks.is_empty();
        for file in files {
            let said = self.write_attachment(&file.name, &file.content_type, file.data.as_slice(), &file.want);
            if let Some(said) = said {
                self.view.attachment_done(said);
            }
        }
        for ask in asks {
            let read = read_for_attach(&ask.path, ask.already, &self.attachments.places);
            self.view.file_attached(read);
        }
        any
    }

    /// Writes one attachment and returns what the footer says now; `None` when
    /// the system was asked to open it, whose answer says it later.
    fn write_attachment(&mut self, name: &str, content_type: &str, data: &[u8], want: &Want) -> Option<String> {
        let size = human_size(data.len());
        if let Want::SaveAs(typed) = want {
            return Some(self.save_as(name, content_type, data, typed, &size));
        }
        let Some(store) = &self.attachments.store else {
            return Some(format!("{name} was not saved: this run keeps no data folder to save it in"));
        };
        let saved = match store.save(name, content_type, data) {
            Ok(saved) => saved,
            Err(error) => return Some(format!("{name} was not saved: {}", plain_words(&error))),
        };
        let dir = store.dir().display().to_string();
        if *want != Want::Open {
            return Some(format!("Saved {} ({size}) in {dir}", saved.name));
        }
        if is_never_opened(&saved.name, content_type) {
            return Some(format!(
                "{} ({size}) was saved but not opened: it is a program, a script or a page that runs code. It is in {dir}",
                saved.name
            ));
        }
        self.attachments.opening = Some(Opening { name: saved.name.clone(), size, dir });
        self.links.open_attachment(&saved.path);
        None
    }

    /// Saves at the place the user typed: a folder that is there (the file goes
    /// in it under its own name) or a new file name in a folder that is there.
    fn save_as(&self, name: &str, content_type: &str, data: &[u8], typed: &str, size: &str) -> String {
        let path = match resolve(typed, self.attachments.places.home.as_deref()) {
            Ok(path) => path,
            Err(why) => return format!("{name} was not saved: {why}"),
        };
        // The user's own folder is never made for them: it must be there.
        let into_folder = path.is_dir() || typed.trim_end().ends_with(['/', '\\']);
        let result = if into_folder {
            save_into(&path, name, content_type, data).map(|saved| saved.path)
        } else {
            save_to(&path, data).map(|()| path.clone())
        };
        match result {
            Ok(at) => {
                let file = at.file_name().map(|each| each.to_string_lossy().into_owned()).unwrap_or_default();
                let folder = at.parent().map(|each| each.display().to_string()).unwrap_or_default();
                format!("Saved {name} ({size}) as {file} in {folder}")
            }
            Err(error) => format!("{name} was not saved: {} ({})", plain_words(&error), path.display()),
        }
    }
}

#[cfg(test)]
include!("attachments.tests.rs");
