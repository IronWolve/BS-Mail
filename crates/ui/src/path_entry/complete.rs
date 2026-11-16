//! The matches for a path being typed: the folders and files of the folder the
//! typed text names that start with what follows its last separator. A path
//! that is not complete starts from the home folder, and `~` stands for it
//! (the same rules the program reads a typed path by). Folders come first;
//! hidden names (those starting with a dot) show only when a dot is typed.
//! Only the folder's own listing is read, never the files, and a folder of
//! tens of thousands of names is cut short, so a keystroke is never slow.

use std::fs;
use std::path::{Path, PathBuf};

/// The most names looked at in one folder.
const MAX_SCAN: usize = 5000;

/// The most matches kept.
const MAX_MATCHES: usize = 200;

/// One folder or file that matches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// The name as the folder lists it.
    pub name: String,
    pub is_dir: bool,
    /// What the box holds once this match is taken: the typed folder part, the
    /// name, and a separator after a folder.
    pub full: String,
}

/// The user's home folder, when the system says where it is.
pub fn home_dir() -> Option<PathBuf> {
    let from = |name: &str| std::env::var_os(name).filter(|value| !value.is_empty()).map(PathBuf::from);
    from("HOME").or_else(|| from("USERPROFILE"))
}

fn is_separator(ch: char) -> bool {
    ch == '/' || (cfg!(windows) && ch == '\\')
}

/// The folder part of `typed` (up to and including its last separator) and
/// the name being typed after it.
fn split(typed: &str) -> (&str, &str) {
    match typed.rfind(is_separator) {
        Some(at) => typed.split_at(at + 1),
        None => ("", typed),
    }
}

/// The folder a folder part names.
fn folder_of(folder_part: &str, home: Option<&Path>) -> Option<PathBuf> {
    if folder_part.is_empty() {
        return home.map(Path::to_path_buf);
    }
    if let Some(rest) = folder_part.strip_prefix('~') {
        return Some(home?.join(rest.trim_start_matches(is_separator)));
    }
    let path = Path::new(folder_part);
    if path.is_absolute() { Some(path.to_path_buf()) } else { Some(home?.join(path)) }
}

/// The folders and files that match `typed`, folders first, each group in
/// alphabetical order, capital letters ignored.
pub fn matches(typed: &str, home: Option<&Path>) -> Vec<Match> {
    if typed == "~" {
        return vec![Match { name: "~".to_owned(), is_dir: true, full: "~/".to_owned() }];
    }
    let (folder_part, prefix) = split(typed);
    let Some(folder) = folder_of(folder_part, home) else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(&folder) else {
        return Vec::new();
    };
    let wanted = prefix.to_lowercase();
    let show_hidden = prefix.starts_with('.');
    let mut found: Vec<Match> = Vec::new();
    for entry in entries.flatten().take(MAX_SCAN) {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if (name.starts_with('.') && !show_hidden) || !name.to_lowercase().starts_with(&wanted) {
            continue;
        }
        let is_dir = entry.file_type().is_ok_and(|kind| kind.is_dir() || (kind.is_symlink() && fs::metadata(entry.path()).is_ok_and(|meta| meta.is_dir())));
        let separator = if is_dir { "/" } else { "" };
        found.push(Match { full: format!("{folder_part}{name}{separator}"), name, is_dir });
    }
    found.sort_by_cached_key(|found| (!found.is_dir, found.name.to_lowercase()));
    found.truncate(MAX_MATCHES);
    found
}

/// The longest start every match's `full` text has in common, which Tab adds
/// to what was typed.
pub fn common_start(found: &[Match]) -> String {
    let Some(first) = found.first() else {
        return String::new();
    };
    let mut end = first.full.len();
    for other in &found[1..] {
        end = end.min(first.full.bytes().zip(other.full.bytes()).take_while(|(a, b)| a == b).count());
    }
    while !first.full.is_char_boundary(end) {
        end -= 1;
    }
    first.full[..end].to_owned()
}

#[cfg(test)]
include!("complete.tests.rs");
