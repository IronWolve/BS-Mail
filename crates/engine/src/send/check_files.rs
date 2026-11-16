//! Looking at the files that go with a message before any of it is built:
//! how many, what each is called, and whether they fit. A name with a line
//! break, another control character, a hidden formatting character or a
//! folder part is refused, never cleaned up (the screen has already made the
//! name of a file safe, so a name that reaches here is a mistake worth
//! saying so).

use model::OutgoingAttachment;

use super::check::{Refusal, is_forbidden, is_hidden_format, shown};
use super::limits::{MAX_FILES, MAX_FILE_NAME_CHARS, MAX_MESSAGE_BYTES};

/// Checks every file and that the text and the files together fit in a
/// message. `text_bytes` is the size of the text so far.
pub fn check_files(files: &[OutgoingAttachment], text_bytes: usize) -> Result<(), Refusal> {
    if files.len() > MAX_FILES {
        return Err(Refusal::new(format!(
            "{} files are attached; the most a message takes is {MAX_FILES}, and nothing was sent",
            files.len()
        )));
    }
    for file in files {
        check_name(&file.name)?;
    }
    let mut total = u64::try_from(text_bytes).unwrap_or(u64::MAX);
    for file in files {
        total = total.saturating_add(model::wire_size(file.data.len() as u64));
    }
    if total > u64::try_from(MAX_MESSAGE_BYTES).unwrap_or(u64::MAX) {
        return Err(Refusal::new(format!(
            "the message with its files is about {} MB once they are encoded for sending; the most this program sends is {} MB, \
             and nothing was sent",
            model::attachments::megabytes(total),
            model::attachments::megabytes(MAX_MESSAGE_BYTES as u64),
        )));
    }
    Ok(())
}

fn check_name(name: &str) -> Result<(), Refusal> {
    let which = shown(name);
    if name.trim().is_empty() {
        return Err(Refusal::new("an attached file has no name; nothing was sent"));
    }
    if name.chars().any(is_forbidden) {
        return Err(Refusal::new(format!(
            "the name of the attached file \"{which}\" contains a line break or another control character; nothing was sent"
        )));
    }
    if name.chars().any(is_hidden_format) {
        return Err(Refusal::new(format!(
            "the name of the attached file \"{which}\" contains a hidden formatting character; nothing was sent"
        )));
    }
    if name.contains('/') || name.contains('\\') {
        return Err(Refusal::new(format!(
            "the name of the attached file \"{which}\" has a folder in it; nothing was sent"
        )));
    }
    if name.chars().count() > MAX_FILE_NAME_CHARS {
        return Err(Refusal::new(format!(
            "the name of the attached file \"{which}\" is longer than {MAX_FILE_NAME_CHARS} characters; nothing was sent"
        )));
    }
    Ok(())
}

#[cfg(test)]
include!("check_files.tests.rs");
