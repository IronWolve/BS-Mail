//! The things on an Options page that take the keyboard focus, and the parts
//! of them a key or a click can press.

use super::layout_choices::{Column, Panel};

/// One stop on a page for the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    /// The list of accounts; Up and Down move its highlight.
    AccountList,
    SwitchAccount,
    EditAccount,
    RemoveAccount,
    AddAccount,
    /// The two buttons of the removal question.
    ConfirmRemove,
    KeepAccount,
    Theme,
    Icons,
    TextSize,
    LineSpacing,
    /// Where the reader sits, and how the sidebar is spaced.
    ReaderPosition,
    Density,
    /// Threads in the list, on or off.
    Threaded,
    /// A column of the list, or a panel, shown or hidden.
    Column(Column),
    Panel(Panel),
    Reload,
}

/// How a control is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    List,
    /// A choice that cycles with Left and Right.
    Choice,
    /// A number with a minus and a plus.
    Stepper,
    Button,
}

impl Control {
    pub fn kind(self) -> Kind {
        match self {
            Self::AccountList => Kind::List,
            Self::Theme | Self::Icons | Self::ReaderPosition | Self::Density | Self::Threaded | Self::Column(_) | Self::Panel(_) => Kind::Choice,
            Self::TextSize | Self::LineSpacing => Kind::Stepper,
            _ => Kind::Button,
        }
    }
}

/// The part of a control that was pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// A button as a whole.
    Whole,
    Prev,
    Next,
    Minus,
    Plus,
    /// A row of the account list, by position.
    Row(usize),
}

#[cfg(test)]
include!("control.tests.rs");
