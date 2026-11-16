//! The ASCII icon set: plain characters, for terminals with no good font. An
//! empty glyph means the plan gives this icon no ASCII form; its slot stays
//! blank.

use super::icon_id::IconId;

/// The raw glyph for `id`, or `""` for a blank slot.
pub fn glyph(id: IconId) -> &'static str {
    match id {
        IconId::Mail | IconId::At => "@",
        IconId::Person => "u",
        IconId::Inbox => "I",
        IconId::Host => "h",
        IconId::Dot | IconId::Star => "*",
        IconId::Ring => "o",
        // Folders and the chart have a name only, no ASCII icon.
        IconId::Folder | IconId::FolderOpen | IconId::Chart => "",
        IconId::FolderOutline => "-",
        IconId::Sent | IconId::Collapsed => ">",
        IconId::Draft => "~",
        IconId::Archive => "A",
        IconId::Bell | IconId::Alert => "!",
        IconId::Ban => "x",
        IconId::Trash => "X",
        IconId::Check => "v",
        IconId::Warning => "!!",
        IconId::Expanded => "v",
        IconId::UpDown => "^v",
        IconId::SortAsc => "^",
        IconId::SortDesc => "v",
        IconId::Attachment => "+",
        IconId::Reply => "<",
        IconId::Forward => ">",
        // Rounded keycap ends exist only in the icon-font set.
        IconId::CapLeft | IconId::CapRight => "",
    }
}

#[cfg(test)]
include!("ascii_set.tests.rs");
