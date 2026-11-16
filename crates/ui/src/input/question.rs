//! What a key or a click says to a two-button question: `y` does it, `n` or
//! Escape keeps things as they are, the arrows and Tab move between the two
//! buttons, Enter does what the focused button says, and a click on a button
//! picks it. The question before a delete and the questions before a folder
//! goes for good all read their events through this.

use surface::{ButtonState, InputEvent, KeyEvent, LogicalKey, NamedKey, PointerButton, Rect};

use crate::delete_prompt::Choice;
use crate::delete_prompt::draw::{button_at, prompt_rects_for};

/// What one event does to a question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Step {
    /// A button was picked: do what it says.
    Pick(Choice),
    /// Focus moves to the other button.
    Toggle,
}

/// The key that answers no: `n` and Escape.
pub(super) fn refuses(key: &KeyEvent) -> bool {
    !key.modifiers.ctrl
        && !key.modifiers.alt
        && !key.modifiers.meta
        && matches!(key.key, LogicalKey::Char('n' | 'N') | LogicalKey::Named(NamedKey::Escape))
}

/// What `event` does to a question whose buttons read `confirm` and `keep`
/// and whose focus is on `focus`, drawn on `area`; `None` for an event it
/// ignores.
pub(super) fn question_step(event: &InputEvent, area: Rect, (confirm, keep): (&str, &str), focus: Choice) -> Option<Step> {
    match event {
        InputEvent::Key(key) => key_step(key, focus),
        InputEvent::Button { button: PointerButton::Left, state: ButtonState::Pressed, pos, .. } => {
            let rects = prompt_rects_for(area, confirm, keep)?;
            button_at(&rects, pos.col, pos.row).map(Step::Pick)
        }
        _ => None,
    }
}

fn key_step(key: &KeyEvent, focus: Choice) -> Option<Step> {
    if key.modifiers.ctrl || key.modifiers.alt || key.modifiers.meta {
        return None;
    }
    Some(match key.key {
        LogicalKey::Char('y' | 'Y') => Step::Pick(Choice::Delete),
        LogicalKey::Char('n' | 'N') | LogicalKey::Named(NamedKey::Escape) => Step::Pick(Choice::Keep),
        LogicalKey::Named(NamedKey::Enter) => Step::Pick(focus),
        LogicalKey::Named(NamedKey::Left | NamedKey::Right | NamedKey::Tab) => Step::Toggle,
        _ => return None,
    })
}

#[cfg(test)]
include!("question.tests.rs");
