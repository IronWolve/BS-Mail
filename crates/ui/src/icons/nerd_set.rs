//! The icon-font set: private-use codepoints from the plan's glyph table.
//! The embedded icon font carries every one of them.

use super::icon_id::IconId;

/// The raw glyph for `id`. The icon set fits it to the two-cell slot.
pub fn glyph(id: IconId) -> &'static str {
    match id {
        IconId::Mail => "\u{f003}",
        IconId::Person => "\u{f2c0}",
        IconId::Inbox => "\u{f01c}",
        IconId::Host => "\u{f108}",
        IconId::Dot => "\u{f111}",
        IconId::Ring => "\u{f10c}",
        IconId::Folder => "\u{f07b}",
        IconId::FolderOpen => "\u{f07c}",
        IconId::FolderOutline => "\u{f114}",
        IconId::Star => "\u{f005}",
        IconId::Sent => "\u{f1d8}",
        IconId::Draft => "\u{f040}",
        IconId::Archive => "\u{f187}",
        IconId::Bell => "\u{f0a2}",
        IconId::Ban => "\u{f05e}",
        IconId::Trash => "\u{f014}",
        IconId::At => "\u{f1fa}",
        IconId::Chart => "\u{f080}",
        IconId::Check => "\u{f00c}",
        IconId::Alert => "\u{f12a}",
        IconId::Warning => "\u{f071}",
        IconId::Expanded => "\u{f078}",
        IconId::Collapsed => "\u{f054}",
        // Plain Unicode in every set, as the plan says.
        IconId::UpDown => "\u{2191}\u{2193}",
        IconId::SortAsc => "\u{f0de}",
        IconId::SortDesc => "\u{f0dd}",
        IconId::Attachment => "\u{f0c6}",
        IconId::CapLeft => "\u{e0b6}",
        IconId::CapRight => "\u{e0b4}",
        IconId::Reply => "\u{f112}",
        IconId::Forward => "\u{f064}",
    }
}

#[cfg(test)]
include!("nerd_set.tests.rs");
