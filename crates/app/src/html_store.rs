//! The safe copies of HTML messages that are handed to the browser: each is a
//! file in `html/` in the data folder, in a folder only its owner can open and
//! readable by its owner only, named from a counter (`mail-000001.html`) and
//! never from anything in the mail. The files an earlier run left are removed
//! when the store is opened, and when more than `KEEP` are there the oldest go,
//! so the folder never grows. Only files named this way are ever removed.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// The folder inside the data folder that holds the pages.
pub const HTML_DIR_NAME: &str = "html";

/// How many pages are kept; a new one past this removes the oldest.
pub const KEEP: usize = 20;

const PREFIX: &str = "mail-";
const SUFFIX: &str = ".html";

/// The number in a page's name, when `name` is one of ours: `mail-` and
/// digits and `.html`, nothing else.
pub fn page_number(name: &str) -> Option<u64> {
    let digits = name.strip_prefix(PREFIX)?.strip_suffix(SUFFIX)?;
    if digits.is_empty() || digits.len() > 18 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

pub struct HtmlStore {
    dir: PathBuf,
    next: u64,
}

impl HtmlStore {
    /// The store for the data folder `data`. The pages an earlier run left are
    /// removed; nothing is made until a page is written.
    pub fn open(data: &Path) -> Self {
        let store = Self { dir: data.join(HTML_DIR_NAME), next: 1 };
        for (_, path) in store.pages() {
            let _ = fs::remove_file(path);
        }
        store
    }

    /// Writes `html` as a new page and returns where it is. The page is a new
    /// file: one that is already there is never written over.
    pub fn write(&mut self, html: &str) -> io::Result<PathBuf> {
        crate::private_dirs::make(&self.dir)?;
        for _ in 0..1000 {
            let path = self.dir.join(format!("{PREFIX}{:06}{SUFFIX}", self.next));
            self.next += 1;
            match open_private(&path) {
                Ok(mut file) => {
                    file.write_all(html.as_bytes())?;
                    file.sync_all()?;
                    self.prune();
                    return Ok(path);
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(io::ErrorKind::AlreadyExists, "no free name for a page"))
    }

    /// The pages in the folder, oldest first, as (number, path).
    fn pages(&self) -> Vec<(u64, PathBuf)> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut pages: Vec<(u64, PathBuf)> = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| !kind.is_dir()))
            .filter_map(|entry| Some((page_number(entry.file_name().to_str()?)?, entry.path())))
            .collect();
        pages.sort();
        pages
    }

    /// Removes the oldest pages while more than `KEEP` are there.
    fn prune(&self) {
        let pages = self.pages();
        let extra = pages.len().saturating_sub(KEEP);
        for (_, path) in pages.into_iter().take(extra) {
            let _ = fs::remove_file(path);
        }
    }
}

#[cfg(unix)]
fn open_private(path: &Path) -> io::Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)
}

#[cfg(not(unix))]
fn open_private(path: &Path) -> io::Result<fs::File> {
    fs::OpenOptions::new().write(true).create_new(true).open(path)
}

#[cfg(test)]
include!("html_store.tests.rs");
