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
}

#[cfg(test)]
include!("focus.tests.rs");
