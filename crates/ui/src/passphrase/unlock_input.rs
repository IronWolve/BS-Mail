//! Keys, typed text and clicks into the start-up prompt.
//!
//! A letter key arrives twice from a window and a terminal: once as a key and
//! once as the text it types. Only the text is typed, so every layout works;
//! keys here are the ones with no text. Escape quits (and, once "Forgot the
//! passphrase" is asking its second question, goes back). Nothing is ever
//! copied out of the prompt.

use surface::{ButtonState, InputEvent, KeyEvent, LogicalKey, NamedKey, PointerButton, PointerShape, Rect};

use super::frame::Focus;
use super::layout::{ButtonId, FieldId, place};
use super::unlock::{EMPTY, UnlockStage, UnlockStep, UnlockView};

/// The places the keyboard goes while the passphrase is asked for.
const RING: [Option<ButtonId>; 4] = [None, Some(ButtonId::Unlock), Some(ButtonId::Forgot), Some(ButtonId::Quit)];
/// And on the second question.
const CONFIRM_RING: [ButtonId; 2] = [ButtonId::Back, ButtonId::SetAside];

impl UnlockView {
    /// What the program is to do after `event` on a grid of `grid` cells.
    pub fn handle_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> UnlockStep {
        let paired = self.paired.take();
        let area = Rect::new(0, 0, grid.0, grid.1);
        let drawn = place(area, &self.prompt()).is_some();
        match event {
            InputEvent::Key(key) if !drawn => if is_escape(key) { UnlockStep::Quit } else { UnlockStep::Nothing },
            InputEvent::Key(key) => self.key(key),
            InputEvent::Text(text) | InputEvent::Paste(text) if drawn => self.text(text, paired),
            InputEvent::Button { button: PointerButton::Left, state: ButtonState::Pressed, pos, .. } if drawn => {
                self.click(area, pos.col, pos.row)
            }
            _ => UnlockStep::Nothing,
        }
    }

    /// The pointer over the prompt: a hand over what a click does.
    pub fn pointer_shape(&self, col: u16, row: u16, grid: (u16, u16)) -> PointerShape {
        let area = Rect::new(0, 0, grid.0, grid.1);
        match place(area, &self.prompt()) {
            Some(placed) if placed.button_at(col, row).is_some() || placed.field_at(col, row).is_some() => PointerShape::Hand,
            _ => PointerShape::Arrow,
        }
    }

    fn key(&mut self, key: &KeyEvent) -> UnlockStep {
        let held = key.modifiers;
        if held.ctrl && matches!(key.key, LogicalKey::Char('q' | 'c')) {
            return UnlockStep::Quit;
        }
        if held.alt || held.meta {
            return UnlockStep::Nothing;
        }
        match self.stage {
            UnlockStage::Entering => self.entering_key(key),
            UnlockStage::ConfirmForget => self.confirming_key(key),
        }
    }

    fn entering_key(&mut self, key: &KeyEvent) -> UnlockStep {
        let held = key.modifiers;
        match key.key {
            LogicalKey::Named(NamedKey::Escape) => UnlockStep::Quit,
            LogicalKey::Named(NamedKey::Enter) => self.activate(),
            LogicalKey::Named(NamedKey::Tab) => self.step(!held.shift),
            LogicalKey::Named(NamedKey::Down) => self.step(true),
            LogicalKey::Named(NamedKey::Up) => self.step(false),
            LogicalKey::Named(NamedKey::Right) if self.focus.button.is_some() => self.step(true),
            LogicalKey::Named(NamedKey::Left) if self.focus.button.is_some() => self.step(false),
            LogicalKey::Named(NamedKey::Backspace) => self.edit(|line| line.backspace()),
            LogicalKey::Char('u') if held.ctrl => self.edit(|line| {
                let had = !line.is_empty();
                line.clear();
                had
            }),
            _ => UnlockStep::Nothing,
        }
    }

    fn confirming_key(&mut self, key: &KeyEvent) -> UnlockStep {
        let held = key.modifiers;
        match key.key {
            LogicalKey::Named(NamedKey::Escape) | LogicalKey::Char('n' | 'N') => {
                self.paired = key_char(key);
                self.back()
            }
            LogicalKey::Char('y' | 'Y') => {
                self.paired = key_char(key);
                UnlockStep::Forget
            }
            LogicalKey::Named(NamedKey::Enter) => self.activate(),
            LogicalKey::Named(NamedKey::Tab) => self.step(!held.shift),
            LogicalKey::Named(NamedKey::Left | NamedKey::Up) => self.step(false),
            LogicalKey::Named(NamedKey::Right | NamedKey::Down) => self.step(true),
            _ => UnlockStep::Nothing,
        }
    }

    /// Typed or pasted text goes into the passphrase, whatever has focus; the
    /// text of a command key that was just handled does not.
    fn text(&mut self, text: &str, paired: Option<char>) -> UnlockStep {
        if self.stage != UnlockStage::Entering {
            return UnlockStep::Nothing;
        }
        let mut letters = text.chars();
        if let (Some(only), None) = (letters.next(), letters.next())
            && paired == Some(only)
        {
            return UnlockStep::Nothing;
        }
        self.focus = Focus { field: Some(FieldId::First), button: None };
        self.problem = None;
        if self.line.push_str(text) { UnlockStep::Redraw } else { UnlockStep::Nothing }
    }

    /// Edits the passphrase with `change`, which says whether it did anything.
    fn edit(&mut self, change: impl FnOnce(&mut super::line::SecretLine) -> bool) -> UnlockStep {
        self.focus = Focus { field: Some(FieldId::First), button: None };
        let had_problem = self.problem.take().is_some();
        if change(&mut self.line) || had_problem { UnlockStep::Redraw } else { UnlockStep::Nothing }
    }

    /// Moves the keyboard one stop round the ring of the question showing.
    fn step(&mut self, forward: bool) -> UnlockStep {
        let count = if self.stage == UnlockStage::Entering { RING.len() } else { CONFIRM_RING.len() };
        let at = self.ring_position();
        let to = if forward { (at + 1) % count } else { (at + count - 1) % count };
        self.focus = match self.stage {
            UnlockStage::Entering => Focus { field: RING[to].is_none().then_some(FieldId::First), button: RING[to] },
            UnlockStage::ConfirmForget => Focus { field: None, button: Some(CONFIRM_RING[to]) },
        };
        UnlockStep::Redraw
    }

    fn ring_position(&self) -> usize {
        match self.stage {
            UnlockStage::Entering => RING.iter().position(|stop| *stop == self.focus.button).unwrap_or(0),
            UnlockStage::ConfirmForget => CONFIRM_RING.iter().position(|stop| Some(*stop) == self.focus.button).unwrap_or(0),
        }
    }

    /// What Enter does where the keyboard is.
    fn activate(&mut self) -> UnlockStep {
        match self.focus.button {
            None | Some(ButtonId::Unlock) => self.submit(),
            Some(button) => self.press(button),
        }
    }

    fn press(&mut self, button: ButtonId) -> UnlockStep {
        match button {
            ButtonId::Unlock => self.submit(),
            ButtonId::Forgot => {
                self.problem = None;
                self.stage = UnlockStage::ConfirmForget;
                // The safe answer has the keyboard.
                self.focus = Focus { field: None, button: Some(ButtonId::Back) };
                UnlockStep::Redraw
            }
            ButtonId::Quit => UnlockStep::Quit,
            ButtonId::SetAside => UnlockStep::Forget,
            ButtonId::Back => self.back(),
            _ => UnlockStep::Nothing,
        }
    }

    fn back(&mut self) -> UnlockStep {
        self.stage = UnlockStage::Entering;
        self.focus = Focus { field: Some(FieldId::First), button: None };
        UnlockStep::Redraw
    }

    fn submit(&mut self) -> UnlockStep {
        if self.line.is_empty() {
            self.problem = Some(EMPTY.to_string());
            return UnlockStep::Redraw;
        }
        self.problem = None;
        UnlockStep::Unlock(self.line.to_secret())
    }

    fn click(&mut self, area: Rect, col: u16, row: u16) -> UnlockStep {
        let Some(placed) = place(area, &self.prompt()) else {
            return UnlockStep::Nothing;
        };
        if let Some(button) = placed.button_at(col, row) {
            self.focus = Focus { field: None, button: Some(button) };
            return self.press(button);
        }
        if self.stage == UnlockStage::Entering && placed.field_at(col, row).is_some() {
            self.focus = Focus { field: Some(FieldId::First), button: None };
            return UnlockStep::Redraw;
        }
        UnlockStep::Nothing
    }
}

fn is_escape(key: &KeyEvent) -> bool {
    let held = key.modifiers;
    !held.ctrl && !held.alt && !held.meta && key.key == LogicalKey::Named(NamedKey::Escape)
}

/// The letter of a key that has one.
fn key_char(key: &KeyEvent) -> Option<char> {
    match key.key {
        LogicalKey::Char(ch) => Some(ch),
        LogicalKey::Named(_) => None,
    }
}

#[cfg(test)]
include!("unlock_input.tests.rs");
