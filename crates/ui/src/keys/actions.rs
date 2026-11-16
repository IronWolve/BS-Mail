//! The actions a key can be bound to, by name: the one list the `[keys]`
//! table, the help box and the key bar all read. A name is what the user
//! writes in `config.toml`; `--print-config` prints every name with the key it
//! has now.

use surface::NamedKey;

use super::spec::KeySpec;
use crate::input::Action;
use crate::view_state::Focus;

/// One action that can be rebound.
#[derive(Debug, Clone, Copy)]
pub struct Rebindable {
    pub name: &'static str,
    pub action: Action,
    /// The keys it has until the table says others. Writing a key for the
    /// action replaces all of these.
    pub keys: &'static [KeySpec],
    /// Keys that always do it as well, whatever the table says.
    pub fixed: &'static [KeySpec],
}

const fn bind(name: &'static str, action: Action, keys: &'static [KeySpec]) -> Rebindable {
    Rebindable { name, action, keys, fixed: &[] }
}

const fn with_fixed(mut rebindable: Rebindable, fixed: &'static [KeySpec]) -> Rebindable {
    rebindable.fixed = fixed;
    rebindable
}

/// Every action that can be rebound, in the order they are listed.
pub const REBINDABLE: &[Rebindable] = &[
    with_fixed(bind("compose", Action::Compose, &[KeySpec::ch('c')]), &[KeySpec::named(NamedKey::Function(2))]),
    bind("reply", Action::Reply, &[KeySpec::ch('r')]),
    bind("reply_all", Action::ReplyAll, &[KeySpec::ch('R')]),
    bind("forward", Action::Forward, &[KeySpec::ch('f')]),
    bind("edit_as_new", Action::EditAsNew, &[KeySpec::ch('w')]),
    with_fixed(bind("delete", Action::Delete, &[KeySpec::ch('d')]), &[KeySpec::named(NamedKey::Delete)]),
    bind("archive", Action::Archive, &[KeySpec::ch('e')]),
    bind("spam", Action::Spam, &[KeySpec::ch('!')]),
    bind("mark_all", Action::MarkAll, &[KeySpec::ch('a')]),
    bind("unmark_all", Action::UnmarkAll, &[KeySpec::ch('A')]),
    bind("invert_marks", Action::InvertMarks, &[KeySpec::ch('i')]),
    bind("next_message", Action::NextMessage, &[KeySpec::ch('n')]),
    bind("previous_message", Action::PreviousMessage, &[KeySpec::ch('p')]),
    bind("next_unread", Action::NextUnread, &[KeySpec::ch('N')]),
    bind("read", Action::ToggleRead, &[KeySpec::ch('u')]),
    bind("star", Action::ToggleStar, &[KeySpec::ch('*')]),
    bind("alert", Action::ToggleAlert, &[KeySpec::ch('+')]),
    bind("tag", Action::Tag, &[KeySpec::ch('t')]),
    bind("move", Action::Move, &[KeySpec::ch('m')]),
    bind("search", Action::Search, &[KeySpec::ch('/')]),
    bind("sync", Action::Sync, &[KeySpec::ch('s')]),
    bind("help", Action::Help, &[KeySpec::ch('?')]),
    bind("options", Action::Options, &[KeySpec::named(NamedKey::Function(10))]),
    bind("go_to", Action::GoTo, &[KeySpec::ch('g')]),
    bind("folder_actions", Action::FolderKeys, &[KeySpec::ch('F')]),
    with_fixed(
        bind("context_menu", Action::ContextMenu, &[KeySpec::ch('.')]),
        &[KeySpec::named(NamedKey::Menu), KeySpec::named(NamedKey::Function(10)).shifted()],
    ),
    bind("copy", Action::Copy, &[KeySpec::ch('y')]),
    bind("copy_sender", Action::CopySender, &[KeySpec::ch('Y')]),
    bind("headers", Action::ViewHeaders, &[KeySpec::ch('H')]),
    bind("source", Action::ViewSource, &[KeySpec::ch('v')]),
    bind("browser", Action::OpenInBrowser, &[KeySpec::ch('b')]),
    bind("next_attachment", Action::NextAttachment, &[KeySpec::ch('x')]),
    bind("previous_attachment", Action::PreviousAttachment, &[KeySpec::ch('X')]),
    bind("sort", Action::SortNext, &[KeySpec::ch('o')]),
    bind("sort_reverse", Action::SortReverse, &[KeySpec::ch('O')]),
    bind("threads", Action::ToggleThreads, &[KeySpec::ch('T')]),
    bind("down", Action::Down, &[KeySpec::ch('j')]),
    bind("up", Action::Up, &[KeySpec::ch('k')]),
    bind("focus_sidebar", Action::FocusPane(Focus::Sidebar), &[KeySpec::ch('1')]),
    bind("focus_list", Action::FocusPane(Focus::List), &[KeySpec::ch('2')]),
    bind("focus_reader", Action::FocusPane(Focus::Reader), &[KeySpec::ch('3')]),
];

/// Names that are known and cannot be changed, with why. A `[keys]` entry for
/// one of these is reported by that reason rather than as an unknown name.
pub const NOT_REBINDABLE: &[(&str, &str)] = &[
    ("quit", "q and Ctrl+Q always quit"),
    ("font_larger", "Ctrl+= is the window's own key"),
    ("font_smaller", "Ctrl+- is the window's own key"),
    ("font_reset", "Ctrl+0 is the window's own key"),
    ("full_screen", "F11 is the window's own key"),
];

/// The place of `name` in `REBINDABLE`.
pub fn index_of(name: &str) -> Option<usize> {
    REBINDABLE.iter().position(|entry| entry.name == name)
}

#[cfg(test)]
include!("actions.tests.rs");
