//! The choices of the layout settings on the Appearance page: where the
//! reader sits, how the sidebar is spaced, which columns of the list and which
//! panels show. The page steps through these; the program saves them.

use model::config::{Config, Density, ReaderPosition};

/// A panel of the main screen that can be shown or hidden. The reader's
/// place has its own choice; this is the rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Sidebar,
    Banner,
    Strip,
    KeyBar,
    Footer,
}

impl Panel {
    pub const ALL: [Panel; 5] = [Self::Sidebar, Self::Banner, Self::Strip, Self::KeyBar, Self::Footer];

    pub fn label(self) -> &'static str {
        match self {
            Self::Sidebar => "Sidebar",
            Self::Banner => "Title banner",
            Self::Strip => "Account strip",
            Self::KeyBar => "Key bar",
            Self::Footer => "Footer",
        }
    }

    /// The table of `config.toml` the panel's `show` key is in.
    pub fn table(self) -> &'static str {
        match self {
            Self::Sidebar => "sidebar",
            Self::Banner => "banner",
            Self::Strip => "strip",
            Self::KeyBar => "keybar",
            Self::Footer => "footer",
        }
    }

    pub fn shown(self, config: &Config) -> bool {
        match self {
            Self::Sidebar => config.sidebar.show,
            Self::Banner => config.banner.show,
            Self::Strip => config.strip.show,
            Self::KeyBar => config.key_bar.show,
            Self::Footer => config.footer.show,
        }
    }

    pub fn set(self, config: &mut Config, shown: bool) {
        match self {
            Self::Sidebar => config.sidebar.show = shown,
            Self::Banner => config.banner.show = shown,
            Self::Strip => config.strip.show = shown,
            Self::KeyBar => config.key_bar.show = shown,
            Self::Footer => config.footer.show = shown,
        }
    }
}

/// A column of the message list that can be shown or hidden. The Subject
/// always shows: it is the column that takes the room the others leave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    Num,
    Flag,
    Star,
    From,
    Date,
    Size,
}

/// The columns in the order the list draws them by default.
const CANONICAL: [&str; 7] = ["num", "flag", "star", "from", "subject", "date", "size"];

impl Column {
    pub const ALL: [Column; 6] = [Self::Num, Self::Flag, Self::Star, Self::From, Self::Date, Self::Size];

    /// The name `[list] columns` uses.
    pub fn name(self) -> &'static str {
        match self {
            Self::Num => "num",
            Self::Flag => "flag",
            Self::Star => "star",
            Self::From => "from",
            Self::Date => "date",
            Self::Size => "size",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Num => "Number",
            Self::Flag => "Read flag",
            Self::Star => "Star",
            Self::From => "From",
            Self::Date => "Date",
            Self::Size => "Size",
        }
    }

    pub fn shown(self, config: &Config) -> bool {
        config.list.columns.iter().any(|name| name == self.name())
    }
}

/// `columns` with `column` shown or hidden and every other entry (and the
/// order the user gave) as it was. A column that is added goes after the
/// last one that comes before it in the usual order.
pub fn toggled_columns(columns: &[String], column: Column, shown: bool) -> Vec<String> {
    let name = column.name();
    let present = columns.iter().any(|entry| entry == name);
    if !shown {
        return columns.iter().filter(|entry| *entry != name).cloned().collect();
    }
    if present {
        return columns.to_vec();
    }
    let rank = |entry: &str| CANONICAL.iter().position(|known| *known == entry);
    let mine = rank(name).unwrap_or(CANONICAL.len());
    let at = columns.iter().rposition(|entry| rank(entry).is_some_and(|other| other < mine)).map_or(0, |last| last + 1);
    let mut out = columns.to_vec();
    out.insert(at, name.to_string());
    out
}

/// What a request from the page changes: the program saves it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutChange {
    ReaderPosition(ReaderPosition),
    Density(Density),
    /// Threads in the list: `[list] threaded`.
    Threaded(bool),
    /// The whole list of columns, as it is to be written.
    Columns(Vec<String>),
    Panel(Panel, bool),
}

pub fn reader_name(position: ReaderPosition) -> &'static str {
    match position {
        ReaderPosition::Below => "below",
        ReaderPosition::Right => "right",
        ReaderPosition::Hidden => "hidden",
    }
}

const READER_POSITIONS: [ReaderPosition; 3] = [ReaderPosition::Below, ReaderPosition::Right, ReaderPosition::Hidden];

/// The next or previous place for the reader, going round.
pub fn cycle_reader(position: ReaderPosition, forward: bool) -> ReaderPosition {
    cycle(&READER_POSITIONS, position, forward)
}

pub fn density_name(density: Density) -> &'static str {
    match density {
        Density::Auto => "auto",
        Density::Roomy => "roomy",
        Density::Compact => "compact",
    }
}

const DENSITIES: [Density; 3] = [Density::Auto, Density::Roomy, Density::Compact];

/// The next or previous spacing, going round.
pub fn cycle_density(density: Density, forward: bool) -> Density {
    cycle(&DENSITIES, density, forward)
}

fn cycle<T: Copy + PartialEq>(all: &[T], current: T, forward: bool) -> T {
    let at = all.iter().position(|candidate| *candidate == current).unwrap_or(0);
    let count = all.len();
    all[if forward { (at + 1) % count } else { (at + count - 1) % count }]
}

#[cfg(test)]
include!("layout_choices.tests.rs");
