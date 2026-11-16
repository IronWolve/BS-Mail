//! The number the server gives each part of a message ("2", "1.3"), worked
//! out from the part tree the reader parsed. The reader's own part ids count
//! every part in the order they were read; the server counts the children of
//! each multipart from 1, and a message that is not multipart is the one part
//! "1". An attachment is asked for by that number (`BODY.PEEK[2]`).

use mail_parser::{Message, PartType};

/// How deep the walk goes: nesting beyond this is not numbered, and so its
/// parts cannot be fetched (the walk is a loop, not a recursion, so a hostile
/// message cannot use up the stack).
const MAX_DEPTH: usize = 32;

/// The section of every part of `message`, indexed by the reader's part id;
/// `None` for a part that is itself a multipart or is not reached.
pub fn section_numbers(message: &Message<'_>) -> Vec<Option<String>> {
    let mut found = vec![None; message.parts.len()];
    let Some(root) = message.parts.first() else {
        return found;
    };
    let PartType::Multipart(_) = root.body else {
        found[0] = Some("1".to_owned());
        return found;
    };
    // Each step: a multipart's part id and the number of its own section.
    let mut pending: Vec<(usize, Vec<u32>)> = vec![(0, Vec::new())];
    while let Some((id, path)) = pending.pop() {
        let Some(PartType::Multipart(children)) = message.parts.get(id).map(|part| &part.body) else {
            continue;
        };
        for (position, child) in children.iter().enumerate() {
            let Ok(child) = usize::try_from(*child) else { continue };
            let Some(part) = message.parts.get(child) else { continue };
            let mut numbered = path.clone();
            numbered.push(u32::try_from(position + 1).unwrap_or(u32::MAX));
            if matches!(part.body, PartType::Multipart(_)) {
                if numbered.len() < MAX_DEPTH {
                    pending.push((child, numbered));
                }
            } else if let Some(slot) = found.get_mut(child) {
                *slot = Some(join(&numbered));
            }
        }
    }
    found
}

fn join(path: &[u32]) -> String {
    path.iter().map(u32::to_string).collect::<Vec<_>>().join(".")
}

#[cfg(test)]
include!("attachment_sections.tests.rs");
