//! The Unicode icon set: safe in any UTF-8 terminal. An empty glyph means the
//! plan gives this icon no Unicode form; its slot stays blank.

use super::icon_id::IconId;

/// The raw glyph for `id`, or `""` for a blank slot.
pub fn glyph(id: IconId) -> &'static str {
    match id {
        IconId::Mail => "\u{2709}",
        IconId::Person => "\u{263a}",
        IconId::Inbox => "\u{25a4}",
        IconId::Host => "\u{25a3}",
        IconId::Dot => "\u{25cf}",
        IconId::Ring => "\u{25cb}",
        // Folders and the chart have a name only, no Unicode icon.
        IconId::Folder | IconId::FolderOpen | IconId::Chart => "",
        IconId::FolderOutline => "\u{25a1}",
        IconId::Star => "\u{2605}",
        IconId::Sent => "\u{27a4}",
        IconId::Draft => "\u{270e}",
        IconId::Archive => "\u{25a6}",
        IconId::Bell => "\u{266a}",
        IconId::Ban => "\u{2298}",
        IconId::Trash => "\u{2717}",
        IconId::At => "@",
        IconId::Check => "\u{2713}",
        IconId::Alert => "!",
        IconId::Warning => "\u{26a0}",
        IconId::Expanded => "\u{25be}",
        IconId::Collapsed => "\u{25b8}",
        IconId::UpDown => "\u{2191}\u{2193}",
        IconId::SortAsc => "\u{25b4}",
        IconId::SortDesc => "\u{25be}",
        IconId::Attachment => "\u{2295}",
        // Rounded keycap ends exist only in the icon-font set.
        IconId::CapLeft | IconId::CapRight => "",
    }
}

#[cfg(test)]
include!("unicode_set.tests.rs");
