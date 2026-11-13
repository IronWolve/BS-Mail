//! Per-message flags and a partial change to them.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags {
    pub seen: bool,
    pub flagged: bool,
    pub answered: bool,
    pub draft: bool,
    pub deleted: bool,
    /// The alert marker shown as a yellow `!` in the list.
    pub alert: bool,
}

/// A partial update: `None` leaves a flag as it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FlagChange {
    pub seen: Option<bool>,
    pub flagged: Option<bool>,
    pub answered: Option<bool>,
    pub draft: Option<bool>,
    pub deleted: Option<bool>,
    pub alert: Option<bool>,
}

impl Flags {
    /// Applies the flags that `change` names and leaves the rest alone.
    pub fn apply(&mut self, change: FlagChange) {
        set(&mut self.seen, change.seen);
        set(&mut self.flagged, change.flagged);
        set(&mut self.answered, change.answered);
        set(&mut self.draft, change.draft);
        set(&mut self.deleted, change.deleted);
        set(&mut self.alert, change.alert);
    }
}

fn set(target: &mut bool, value: Option<bool>) {
    if let Some(value) = value {
        *target = value;
    }
}
