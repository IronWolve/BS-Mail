//! IMAP flags and keywords to the model's flags and tags.

use async_imap::types::Flag;
use model::Flags;

/// Keywords the server or other clients set for their own bookkeeping; they
/// are not tags the user sees.
const BOOKKEEPING: [&str; 6] = [
    "nonjunk",
    "junk",
    "notjunk",
    "forwarded",
    "mdnsent",
    "phishing",
];

/// The flags of one message and the free-form keywords it carries.
pub fn map_flags(flags: &[Flag<'_>]) -> (Flags, Vec<String>) {
    let mut out = Flags::default();
    let mut tags = Vec::new();
    for flag in flags {
        match flag {
            Flag::Seen => out.seen = true,
            Flag::Flagged => out.flagged = true,
            Flag::Answered => out.answered = true,
            Flag::Draft => out.draft = true,
            Flag::Deleted => out.deleted = true,
            Flag::Custom(name) => classify_keyword(name, &mut out, &mut tags),
            _ => {}
        }
    }
    (out, tags)
}

/// A system flag the server spelled in another letter case. The IMAP
/// library knows only the exact spelling (`\Seen`) as a system flag and hands
/// the rest over as keywords, but the standard treats the names as the same
/// whatever the case.
fn set_system_flag(name: &str, flags: &mut Flags) -> bool {
    match name.to_ascii_lowercase().as_str() {
        "\\seen" => flags.seen = true,
        "\\flagged" => flags.flagged = true,
        "\\answered" => flags.answered = true,
        "\\draft" => flags.draft = true,
        "\\deleted" => flags.deleted = true,
        _ => return false,
    }
    true
}

fn classify_keyword(name: &str, flags: &mut Flags, tags: &mut Vec<String>) {
    if set_system_flag(name, flags) {
        return;
    }
    let lower = name.to_ascii_lowercase();
    let bare = lower.trim_start_matches(['$', '\\']);
    if bare == "important" {
        flags.alert = true;
    } else if !name.starts_with('$') && !name.starts_with('\\') && !BOOKKEEPING.contains(&bare) {
        tags.push(name.to_owned());
    }
}

#[cfg(test)]
include!("flag_map.tests.rs");
