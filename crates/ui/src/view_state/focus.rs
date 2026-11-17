//! Which region has the keyboard focus.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    Sidebar,
    #[default]
    List,
    Reader,
}

impl Focus {
    /// The next region in reading order, wrapping around.
    pub fn next(self) -> Self {
        match self {
            Self::Sidebar => Self::List,
            Self::List => Self::Reader,
            Self::Reader => Self::Sidebar,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Sidebar => Self::Reader,
            Self::List => Self::Sidebar,
            Self::Reader => Self::List,
        }
    }

    /// As `next`, going over the sidebar when it is not on screen.
    pub fn next_shown(self, sidebar_shown: bool) -> Self {
        match self.next() {
            Self::Sidebar if !sidebar_shown => Self::Sidebar.next(),
            next => next,
        }
    }

    /// As `previous`, going over the sidebar when it is not on screen.
    pub fn previous_shown(self, sidebar_shown: bool) -> Self {
        match self.previous() {
            Self::Sidebar if !sidebar_shown => Self::Sidebar.previous(),
            previous => previous,
        }
    }
}

#[cfg(test)]
include!("focus.tests.rs");
