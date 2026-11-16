//! Every icon a widget can ask for. Widgets name an `IconId`, never a glyph.

/// One icon of the plan's table. The active icon set decides what is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IconId {
    /// Banner, list title and reader title envelope.
    Mail,
    /// Account strip: account.
    Person,
    /// Account strip mailbox and the Inbox folder.
    Inbox,
    /// Account strip: host.
    Host,
    /// Online, unread and selected-account dot.
    Dot,
    /// Other account, open-item bullet.
    Ring,
    /// A closed folder (a folder that is not expanded).
    Folder,
    /// An open folder: the Folders section title and an expanded folder.
    FolderOpen,
    /// A child folder.
    FolderOutline,
    Star,
    Sent,
    Draft,
    Archive,
    Bell,
    Ban,
    Trash,
    /// The Accounts section title.
    At,
    /// The Status section title.
    Chart,
    Check,
    /// The alert flag in the list.
    Alert,
    /// The warning triangle in a subject.
    Warning,
    Expanded,
    Collapsed,
    /// The "Navigate" hint arrows.
    UpDown,
    SortAsc,
    SortDesc,
    Attachment,
    /// Left round end of a keycap.
    CapLeft,
    /// Right round end of a keycap.
    CapRight,
    /// The list's mark for a message that was answered.
    Reply,
    /// The list's mark for a message that was forwarded.
    Forward,
}

impl IconId {
    /// Every icon, in declaration order; `ALL[i].index() == i`.
    pub const ALL: [IconId; 31] = [
        IconId::Mail,
        IconId::Person,
        IconId::Inbox,
        IconId::Host,
        IconId::Dot,
        IconId::Ring,
        IconId::Folder,
        IconId::FolderOpen,
        IconId::FolderOutline,
        IconId::Star,
        IconId::Sent,
        IconId::Draft,
        IconId::Archive,
        IconId::Bell,
        IconId::Ban,
        IconId::Trash,
        IconId::At,
        IconId::Chart,
        IconId::Check,
        IconId::Alert,
        IconId::Warning,
        IconId::Expanded,
        IconId::Collapsed,
        IconId::UpDown,
        IconId::SortAsc,
        IconId::SortDesc,
        IconId::Attachment,
        IconId::CapLeft,
        IconId::CapRight,
        IconId::Reply,
        IconId::Forward,
    ];

    /// Position in `ALL`, for table lookups.
    pub fn index(self) -> usize {
        self as usize
    }
}

#[cfg(test)]
include!("icon_id.tests.rs");
