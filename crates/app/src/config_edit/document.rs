//! Changing one value in the text of a `config.toml`.

use toml_edit::{DocumentMut, Item};

use super::problem::SaveProblem;
use super::setting::Setting;

/// The file's text with `setting` set and everything else as it was.
///
/// An existing value is replaced in place, keeping the spaces and the comment
/// after it; a missing key is added at the end of its table; a missing table
/// is added at the end of the file, after a blank line. The line ends of the
/// file, and whether it ends with one, stay as they were.
pub(super) fn with_setting(text: &str, setting: &Setting) -> Result<String, SaveProblem> {
    let mut document: DocumentMut = text.parse().map_err(|error: toml_edit::TomlError| SaveProblem::NotToml {
        line: error.span().map_or(1, |span| line_of(text, span.start)),
    })?;
    let table = setting.table();
    let Some(item) = document.get_mut(table) else {
        return Ok(with_table_added(text, setting));
    };
    if let Some(only_nested) = item.as_table_mut().filter(|found| found.is_implicit()) {
        // A table that exists only through its sub-tables gets its own header.
        only_nested.set_implicit(false);
    }
    let found = item.as_table_like_mut().ok_or(SaveProblem::NotATable(table))?;
    let mut new = setting.toml_value();
    match found.get_mut(setting.key()).and_then(Item::as_value_mut) {
        Some(old) => {
            *new.decor_mut() = old.decor().clone();
            *old = new;
        }
        None => {
            found.insert(setting.key(), Item::Value(new));
        }
    }
    Ok(with_ends_of(text, document.to_string()))
}

/// `text` with the setting's table and key written after it. Plain text
/// appended to the original, so nothing already in it can move.
fn with_table_added(text: &str, setting: &Setting) -> String {
    let eol = if only_windows_line_ends(text) { "\r\n" } else { "\n" };
    let mut out = text.to_string();
    if !out.is_empty() {
        if !out.ends_with('\n') {
            out.push_str(eol);
        }
        if !out.ends_with("\n\n") && !out.ends_with("\r\n\r\n") {
            out.push_str(eol);
        }
    }
    let value = setting.toml_value().to_string();
    out.push_str(&format!("[{}]{eol}{} = {}{eol}", setting.table(), setting.key(), value.trim()));
    out
}

/// `edited` with the original's line ends: a file written with `\r\n`
/// throughout stays that way, and one with no newline at its very end gets
/// none.
fn with_ends_of(text: &str, edited: String) -> String {
    let mut out = if only_windows_line_ends(text) { edited.replace("\r\n", "\n").replace('\n', "\r\n") } else { edited };
    if !text.is_empty() && !text.ends_with('\n') && out.ends_with('\n') {
        out.pop();
        if out.ends_with('\r') {
            out.pop();
        }
    }
    out
}

fn only_windows_line_ends(text: &str) -> bool {
    text.contains("\r\n") && !text.replace("\r\n", "").contains('\n')
}

/// The line, counting from one, that byte `offset` is on.
fn line_of(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].matches('\n').count() + 1
}

#[cfg(test)]
include!("document.tests.rs");
