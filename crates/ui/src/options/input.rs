//! Keys into the Options screen.
//!
//! One ring of stops: the list of pages, then each control of the page.
//! Tab and Shift+Tab walk it. Up and Down walk it too from a control, and on
//! the list of pages they choose the page; on the list of accounts they move
//! the highlight and walk on at its ends. Left and Right change a choice or a
//! number, Enter or Space press a button, Escape closes (or, inside the
//! removal question, only cancels it).
//!
//! A letter key arrives twice from a window: once as a key and once as the
//! text the layout made of it. Space is read from the text, as the account
//! form reads it; keys here are the ones with no text.

use surface::{KeyEvent, LogicalKey, NamedKey};

use super::control::{Control, Kind, Part};
use super::state::{OptionsState, Stop};

/// What the screen wants done after an input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Not something the screen uses.
    Ignored,
    /// The screen changed; draw it again.
    Changed,
    /// Close the screen.
    Close,
    /// A control was pressed, by key or click; the view acts on it.
    Press(Control, Part),
}

/// What the keys need to know about the page.
pub struct Nav<'a> {
    /// The controls of the page, in Tab order.
    pub controls: &'a [Control],
    /// How many accounts the list holds.
    pub accounts: usize,
}

pub fn handle_key(state: &mut OptionsState, key: &KeyEvent, nav: &Nav<'_>) -> Outcome {
    let held = key.modifiers;
    if held.ctrl || held.alt || held.meta {
        return Outcome::Ignored;
    }
    let LogicalKey::Named(named) = key.key else {
        return Outcome::Ignored;
    };
    match named {
        NamedKey::Escape => escape(state),
        NamedKey::Tab => step(state, !held.shift, nav),
        NamedKey::Down => vertical(state, true, nav),
        NamedKey::Up => vertical(state, false, nav),
        NamedKey::Right => horizontal(state, true, nav),
        NamedKey::Left => horizontal(state, false, nav),
        NamedKey::Enter => activate(state, nav),
        _ => Outcome::Ignored,
    }
}

/// Typed text: a space presses, like Enter. Nothing else is typed here.
pub fn handle_text(state: &mut OptionsState, text: &str, nav: &Nav<'_>) -> Outcome {
    if text == " " { activate(state, nav) } else { Outcome::Ignored }
}

fn escape(state: &mut OptionsState) -> Outcome {
    if state.confirming {
        state.cancel_confirm();
        Outcome::Changed
    } else {
        Outcome::Close
    }
}

fn step(state: &mut OptionsState, forward: bool, nav: &Nav<'_>) -> Outcome {
    let before = state.stop;
    state.step_stop(forward, nav.controls);
    if state.stop == before { Outcome::Ignored } else { Outcome::Changed }
}

fn changed_if(moved: bool) -> Outcome {
    if moved { Outcome::Changed } else { Outcome::Ignored }
}

fn vertical(state: &mut OptionsState, forward: bool, nav: &Nav<'_>) -> Outcome {
    if state.confirming {
        return step(state, forward, nav);
    }
    match state.stop {
        Stop::Pages => changed_if(state.move_page(forward)),
        Stop::Control(Control::AccountList) => {
            if state.move_cursor(forward, nav.accounts) {
                Outcome::Changed
            } else {
                step(state, forward, nav)
            }
        }
        Stop::Control(_) => step(state, forward, nav),
    }
}

fn horizontal(state: &mut OptionsState, forward: bool, nav: &Nav<'_>) -> Outcome {
    if state.confirming {
        return step(state, forward, nav);
    }
    match state.stop {
        Stop::Pages => changed_if(forward && state.enter_page(nav.controls)),
        Stop::Control(control) => match control.kind() {
            Kind::Choice => Outcome::Press(control, if forward { Part::Next } else { Part::Prev }),
            Kind::Stepper => Outcome::Press(control, if forward { Part::Plus } else { Part::Minus }),
            Kind::List | Kind::Button => Outcome::Ignored,
        },
    }
}

fn activate(state: &mut OptionsState, nav: &Nav<'_>) -> Outcome {
    match state.stop {
        Stop::Pages => changed_if(state.enter_page(nav.controls)),
        Stop::Control(control) => match control.kind() {
            Kind::Button => Outcome::Press(control, Part::Whole),
            Kind::Choice => Outcome::Press(control, Part::Next),
            Kind::List | Kind::Stepper => Outcome::Ignored,
        },
    }
}

#[cfg(test)]
include!("input.tests.rs");
