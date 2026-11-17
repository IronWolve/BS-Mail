//! Role names as constants, so a widget never types a role as a bare string.
//!
//! `ALL` lists every role a widget may ask for; a theme must define each one.
//! Three are reserved: no widget asks for them yet (marked `not drawn yet`
//! below), so a theme that changes them changes no frame.
//! A role is named for its place and meaning (`list.selected`), never for its
//! color. A child name (`list.date.recent`) starts from its parent's style.

macro_rules! roles {
    ($($name:ident = $text:literal;)*) => {
        $(pub const $name: &str = $text;)*

        /// Every role name above, in declaration order.
        pub const ALL: &[&str] = &[$($name),*];
    };
}

roles! {
    // Window and frame lines.
    BASE = "base";
    BORDER = "border";
    BORDER_FOCUSED = "border.focused";
    BORDER_HOVER = "border.hover"; // a bar under the pointer
    BORDER_DRAG = "border.drag"; // not drawn yet
    BORDER_RULE = "border.rule";
    SEPARATOR = "separator";

    // Title banner.
    BANNER_SLASHES = "banner.slashes";
    BANNER_ICON = "banner.icon";
    BANNER_NAME = "banner.name";
    BANNER_DIVIDER = "banner.divider";
    BANNER_TAGLINE = "banner.tagline";
    BANNER_VERSION = "banner.version";

    // Account strip.
    STRIP_LABEL = "strip.label";
    STRIP_ICON = "strip.icon";
    STRIP_VALUE = "strip.value";
    STRIP_UNREAD = "strip.unread";
    STRIP_ONLINE = "strip.online";
    STRIP_CONNECTING = "strip.connecting";
    STRIP_OFFLINE = "strip.offline";

    // Sidebar: section titles, folders, accounts, status.
    SECTION_TITLE = "section.title";
    SECTION_ICON = "section.icon";
    SIDEBAR_ITEM = "sidebar.item";
    SIDEBAR_COUNT = "sidebar.count";
    SIDEBAR_CHEVRON = "sidebar.chevron";
    SIDEBAR_SELECTED = "sidebar.selected";
    SIDEBAR_HOVER = "sidebar.hover";
    SIDEBAR_ACCOUNT = "sidebar.account";
    SIDEBAR_ACCOUNT_ACTIVE = "sidebar.account.active";
    SIDEBAR_ACCOUNT_DOT = "sidebar.account.dot";
    SIDEBAR_ACCOUNT_RING = "sidebar.account.ring";
    STATUS_LABEL = "status.label";
    STATUS_VALUE = "status.value";
    STATUS_OK = "status.ok";
    STATUS_COUNT = "status.count";
    GAUGE_FILL = "gauge.fill";
    GAUGE_TRACK = "gauge.track";
    GAUGE_OUTLINE = "gauge.outline";

    // Folder icons.
    ICON_INBOX = "icon.inbox";
    ICON_STARRED = "icon.starred";
    ICON_SENT = "icon.sent";
    ICON_DRAFTS = "icon.drafts";
    ICON_ARCHIVE = "icon.archive";
    ICON_FOLDER = "icon.folder";
    ICON_ALERTS = "icon.alerts";
    ICON_SPAM = "icon.spam";
    ICON_TRASH = "icon.trash";

    // Message list.
    LIST_TITLE = "list.title";
    LIST_ICON = "list.icon";
    LIST_HINT_KEY = "list.hint.key";
    LIST_HINT_LABEL = "list.hint.label";
    LIST_HEADER = "list.header";
    LIST_HEADER_BAR = "list.header.bar";
    LIST_ROW = "list.row";
    LIST_ROW_UNREAD = "list.row.unread"; // not drawn yet
    LIST_ROW_HOVER = "list.row.hover"; // a message row under the pointer
    LIST_SELECTED = "list.selected";
    LIST_MARKED = "list.marked"; // not drawn yet: a marked row reverses its number cell
    LIST_FLAG_UNREAD = "list.flag.unread";
    LIST_FLAG_ALERT = "list.flag.alert";
    LIST_FLAG_ANSWERED = "list.flag.answered";
    LIST_ATTACHMENT = "list.attachment";
    LIST_STAR = "list.star";
    LIST_DATE_RECENT = "list.date.recent";
    LIST_DATE_AHEAD = "list.date.ahead";
    LIST_DATE = "list.date";
    LIST_SIZE = "list.size";
    SUBJECT_TAG_DEFAULT = "subject.tag.default";
    SUBJECT_TAG_PROJECT = "subject.tag.project";
    SUBJECT_TAG_ALERT = "subject.tag.alert";
    SUBJECT_ALERT = "subject.alert";
    SUBJECT_WARNING = "subject.warning";
    SUBJECT_NUMBER = "subject.number";

    // Reader box: title, counter, header block, body.
    READER_TITLE = "reader.title";
    READER_ICON = "reader.icon";
    READER_COUNTER = "reader.counter";
    READER_COUNTER_UNREAD = "reader.counter.unread";
    READER_HEADER_LABEL = "reader.header.label";
    READER_HEADER_VALUE = "reader.header.value";
    READER_HEADER_TAGS = "reader.header.tags";
    BODY_TEXT = "body.text";
    BODY_QUOTE = "body.quote";
    BODY_NAME = "body.name";
    BODY_BULLET_DONE = "body.bullet.done";
    BODY_BULLET_OPEN = "body.bullet.open";
    BODY_LEAD = "body.lead";
    BODY_LEAD_NEXT = "body.lead.next";
    BODY_TOKEN = "body.token";
    BODY_PATH = "body.path";
    BODY_LINK = "body.link";
    BODY_LINK_HOVER = "body.link.hover"; // a link under the pointer
    BODY_LINK_PRESSED = "body.link.pressed"; // a link the button is down on
    BODY_LINK_FOCUS = "body.link.focus"; // the link the keys are on (l and L)
    BODY_SIGNATURE = "body.signature";
    READER_SELECTION = "reader.selection"; // the text selected with the pointer or Shift and the arrows
    READER_FOUND = "reader.found"; // a match of the find box
    READER_FOUND_CURRENT = "reader.found.current"; // the match the find is on

    // Key bar and footer.
    KEYBAR_CAP = "keybar.cap";
    KEYBAR_LABEL = "keybar.label";
    KEYBAR_HOVER = "keybar.hover"; // a key bar button or footer item under the pointer
    FOOTER_SLASHES = "footer.slashes";
    FOOTER_TEXT = "footer.text";
    FOOTER_OK = "footer.ok";
    FOOTER_UNREAD = "footer.unread";
    FOOTER_KEY = "footer.key";
    FOOTER_ERROR = "footer.error";

    // Scrollbars and menus.
    SCROLLBAR_THUMB = "scrollbar.thumb";
    SCROLLBAR_TRACK = "scrollbar.track";
    MENU_BASE = "menu.base";
    MENU_SELECTED = "menu.selected";
    MENU_BORDER = "menu.border";
}

#[cfg(test)]
include!("roles.tests.rs");
