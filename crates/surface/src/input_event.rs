//! Normalized input events: what every host delivers to the UI, whatever the
//! native source. Positions are in cells plus the fraction inside the cell.

/// A pointer position: the cell, and where inside it (0.0 to 1.0 from the
/// top-left). A host that cannot tell reports 0.5 for both fractions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellPosition {
    pub col: u16,
    pub row: u16,
    pub frac_x: f32,
    pub frac_y: f32,
}

impl CellPosition {
    /// The middle of the cell at `col`, `row`.
    pub fn at(col: u16, row: u16) -> Self {
        Self {
            col,
            row,
            frac_x: 0.5,
            frac_y: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    /// Command on macOS, the logo key elsewhere.
    pub meta: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonState {
    Pressed,
    Released,
}

/// Keys that have no character of their own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedKey {
    Enter,
    Escape,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    Menu,
    /// `F1` is `Function(1)`.
    Function(u8),
}

/// The key as the active layout names it; shortcuts match on this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicalKey {
    Char(char),
    Named(NamedKey),
}

/// The key's position on the keyboard, as an opaque host-defined code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalKey(pub u32);

/// A key press. Releases are not delivered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub key: LogicalKey,
    /// `None` where the host cannot tell (a terminal).
    pub physical: Option<PhysicalKey>,
    pub modifiers: Modifiers,
    pub repeat: bool,
}

/// Input method composition, delivered only while a text field has focus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositionEvent {
    Start,
    /// Pre-edit text, with the cursor as a byte range inside it.
    Update {
        text: String,
        cursor: Option<(usize, usize)>,
    },
    Commit(String),
    Cancel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    PointerMoved {
        pos: CellPosition,
        modifiers: Modifiers,
    },
    Button {
        button: PointerButton,
        state: ButtonState,
        pos: CellPosition,
        modifiers: Modifiers,
    },
    /// Scroll in whole rows and columns, positive down and right; the host
    /// accumulates pixel-precise deltas into whole rows.
    Wheel {
        pos: CellPosition,
        rows: i16,
        cols: i16,
        modifiers: Modifiers,
    },
    Key(KeyEvent),
    /// Committed text from the layout, dead keys and compose sequences.
    Text(String),
    Composition(CompositionEvent),
    Paste(String),
    Focus { gained: bool },
    /// The grid changed size.
    Resized { cols: u16, rows: u16 },
    CloseRequested,
}
