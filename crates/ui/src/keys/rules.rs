//! Which keys no action may be given. They are kept out of the `[keys]` table
//! because something else already owns them (the keys of dialogs and text
//! boxes, the window's own shortcuts, the keys that close the program), or
//! because typing would trip over them.
//!
//! A text box, the composer and the menus read every key before the main
//! screen does, so a plain letter rebound to an action never reaches an
//! action while text is typed. What can still go wrong is a key that types
//! something on some keyboards (AltGr is Ctrl and Alt together), or one that a
//! terminal cannot tell from another (Ctrl+i is Tab).

use surface::{LogicalKey, NamedKey};

use super::spec::KeySpec;

/// The keys the program keeps for itself, as the plan lists them: each entry
/// is keys written as the `[keys]` table writes them, and why. No action can be
/// given any of them (a test reads every one back through `refusal`). F2 and
/// the Delete, Menu and Shift+F10 keys are also an action's own extra keys,
/// which another action cannot take either (that is a conflict, not a refusal).
pub const FIXED_KEYS: &[(&str, &str)] = &[
    ("Esc", "closes or goes back"),
    ("Enter, Ctrl+Enter", "goes in, presses, sends"),
    ("Tab, Shift+Tab", "change pane"),
    ("Backspace", "goes back"),
    ("Space, Shift+Space", "pages and marks"),
    ("Up, Down, Left, Right, Home, End, PageUp, PageDown, Shift+Up, Shift+Down", "move, and with Shift select"),
    ("Delete, Insert, Menu, F11, Shift+F10", "delete, the menus and the window's full screen"),
    ("q, Ctrl+q", "quit"),
    ("Ctrl+u, Ctrl+d", "half a page"),
    ("Ctrl+c, Ctrl+v, Ctrl+x, Ctrl+a, Ctrl+z, Ctrl+y", "copy, paste, cut, select all, undo, redo in a text box"),
    ("Ctrl+o, Ctrl+r, Ctrl+w", "attach a file, and the keys kept for reload and layout"),
    ("Ctrl+=, Ctrl+-, Ctrl+0", "the window's text size"),
    ("Ctrl+h, Ctrl+i, Ctrl+j, Ctrl+m, Ctrl+[", "a terminal sends these as Backspace, Tab, Enter and Esc"),
    ("o, O", "kept for sorting, and for opening an attachment"),
];

/// The characters a rule names that no action may be given with Ctrl.
const CTRL_FIXED: &str = "cvxazyoruwdqhijm[=-0";

/// Why `spec` cannot be given to an action, in plain words; `None` when it
/// can. The keys an action already has of its own (F2, Delete, Menu) are
/// not asked about here: they are its.
pub fn refusal(spec: &KeySpec) -> Option<String> {
    let name = spec.text();
    if spec.ctrl && spec.alt {
        return Some(format!("{name} is what AltGr types on some keyboards, so it would take text typed there"));
    }
    match spec.key {
        LogicalKey::Named(named) => named_refusal(spec, named, &name),
        LogicalKey::Char(' ') => Some(format!("{name} is fixed: it pages and marks")),
        LogicalKey::Char(ch) if spec.ctrl && CTRL_FIXED.contains(ch) => Some(format!("{name} is fixed: the program or the terminal keeps it")),
        LogicalKey::Char('q') if !spec.ctrl && !spec.alt => Some(format!("{name} is fixed: it quits")),
        LogicalKey::Char('o' | 'O') if !spec.ctrl && !spec.alt => Some(format!("{name} is kept for sorting and for opening an attachment")),
        LogicalKey::Char(_) => None,
    }
}

fn named_refusal(spec: &KeySpec, named: NamedKey, name: &str) -> Option<String> {
    match named {
        // F10 is the Options key, F2 composes, F11 is the window's full
        // screen; Shift+F10 is the menu key. Which of them an action may take
        // is for the conflict check: an action's own keys are its.
        NamedKey::Function(11) => Some(format!("{name} is fixed: it is the window's full screen")),
        NamedKey::Function(10) if spec.shift && !spec.ctrl && !spec.alt => Some(format!("{name} is fixed: it opens the menu")),
        NamedKey::Function(1..=24) => None,
        _ => Some(format!("{name} is fixed: the keys that move, go in and out and close are not rebindable")),
    }
}

#[cfg(test)]
include!("rules.tests.rs");
