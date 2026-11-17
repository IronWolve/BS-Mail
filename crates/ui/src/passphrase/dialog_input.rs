//! Keys, typed text and clicks into the passphrase dialog. As in the start-up
//! prompt, only the text a key types is typed; the keys here are the ones with
//! no text. Escape closes the dialog and changes nothing.

use surface::{ButtonState, InputEvent, KeyEvent, LogicalKey, NamedKey, PointerButton, PointerShape, Rect};

use super::dialog::{DialogKind, DialogStep, PassphraseDialog};
use super::frame::Focus;
use super::layout::{ButtonId, FieldId, place};
use super::line::SecretLine;

/// The places the keyboard goes while a passphrase is typed.
const SET_RING: [Stop; 4] = [Stop::Field(FieldId::First), Stop::Field(FieldId::Second), Stop::Button(ButtonId::Save), Stop::Button(ButtonId::Cancel)];
/// And on the question before a key file.
const KEY_RING: [Stop; 2] = [Stop::Button(ButtonId::Keep), Stop::Button(ButtonId::Switch)];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stop {
    Field(FieldId),
    Button(ButtonId),
}

impl PassphraseDialog {
    /// What the program is to do after `event` on a grid of `grid` cells.
    pub fn handle_input(&mut self, event: &InputEvent, grid: (u16, u16)) -> DialogStep {
        let paired = self.paired.take();
        let area = Rect::new(0, 0, grid.0, grid.1);
        let drawn = place(area, &self.prompt()).is_some();
        match event {
            InputEvent::Key(key) if !drawn => if is_escape(key) { DialogStep::Cancel } else { DialogStep::Nothing },
            InputEvent::Key(key) => self.key(key),
            InputEvent::Text(text) | InputEvent::Paste(text) if drawn => self.text(text, paired),
            InputEvent::Button { button: PointerButton::Left, state: ButtonState::Pressed, pos, .. } if drawn => {
                self.click(area, pos.col, pos.row)
            }
            _ => DialogStep::Nothing,
        }
    }

    /// The pointer over the dialog: a hand over what a click does.
    pub fn pointer_shape(&self, col: u16, row: u16, grid: (u16, u16)) -> PointerShape {
        match place(Rect::new(0, 0, grid.0, grid.1), &self.prompt()) {
            Some(placed) if placed.button_at(col, row).is_some() || placed.field_at(col, row).is_some() => PointerShape::Hand,
            _ => PointerShape::Arrow,
        }
    }

    fn ring(&self) -> &'static [Stop] {
        if self.kind.is_question() { &KEY_RING } else { &SET_RING }
    }

    fn stop(&self) -> Stop {
        match (self.focus.field, self.focus.button) {
            (Some(field), _) => Stop::Field(field),
            (None, Some(button)) => Stop::Button(button),
            (None, None) => self.ring()[0],
        }
    }

    fn go_to(&mut self, stop: Stop) {
        self.focus = match stop {
            Stop::Field(field) => Focus { field: Some(field), button: None },
            Stop::Button(button) => Focus { field: None, button: Some(button) },
        };
    }

    fn key(&mut self, key: &KeyEvent) -> DialogStep {
        let held = key.modifiers;
        if held.alt || held.meta || (held.ctrl && key.key != LogicalKey::Char('u')) {
            return DialogStep::Nothing;
        }
        if self.kind.is_question() {
            return self.question_key(key);
        }
        match key.key {
            LogicalKey::Named(NamedKey::Escape) => DialogStep::Cancel,
            LogicalKey::Named(NamedKey::Enter) => self.activate(),
            LogicalKey::Named(NamedKey::Tab) => self.step(!held.shift),
            LogicalKey::Named(NamedKey::Down) => self.step(true),
            LogicalKey::Named(NamedKey::Up) => self.step(false),
            LogicalKey::Named(NamedKey::Right) if self.focus.button.is_some() => self.step(true),
            LogicalKey::Named(NamedKey::Left) if self.focus.button.is_some() => self.step(false),
            LogicalKey::Named(NamedKey::Backspace) => self.edit(SecretLine::backspace),
            LogicalKey::Char('u') if held.ctrl => self.edit(|line| {
                let had = !line.is_empty();
                line.clear();
                had
            }),
            _ => DialogStep::Nothing,
        }
    }

    fn question_key(&mut self, key: &KeyEvent) -> DialogStep {
        match key.key {
            LogicalKey::Named(NamedKey::Escape) => DialogStep::Cancel,
            LogicalKey::Char('n' | 'N') => {
                self.paired = Some('n');
                DialogStep::Cancel
            }
            LogicalKey::Char('y' | 'Y') => {
                self.paired = Some('y');
                self.confirmed()
            }
            LogicalKey::Named(NamedKey::Enter) => self.activate(),
            LogicalKey::Named(NamedKey::Tab | NamedKey::Left | NamedKey::Right | NamedKey::Up | NamedKey::Down) => self.step(true),
            _ => DialogStep::Nothing,
        }
    }

    /// Typed or pasted text goes into the box that has focus.
    fn text(&mut self, text: &str, paired: Option<char>) -> DialogStep {
        let mut letters = text.chars();
        if let (Some(only), None) = (letters.next(), letters.next())
            && paired == Some(only)
        {
            return DialogStep::Nothing;
        }
        let added = match self.focus.field {
            Some(FieldId::First) => self.first.push_str(text),
            Some(FieldId::Second) => self.second.push_str(text),
            None => false,
        };
        if added {
            self.problem = None;
        }
        if added { DialogStep::Redraw } else { DialogStep::Nothing }
    }

    /// Edits the box that has focus with `change`, which says whether it did
    /// anything.
    fn edit(&mut self, change: impl FnOnce(&mut SecretLine) -> bool) -> DialogStep {
        let changed = match self.focus.field {
            Some(FieldId::First) => change(&mut self.first),
            Some(FieldId::Second) => change(&mut self.second),
            None => false,
        };
        if changed {
            self.problem = None;
            DialogStep::Redraw
        } else {
            DialogStep::Nothing
        }
    }

    fn step(&mut self, forward: bool) -> DialogStep {
        let ring = self.ring();
        let at = ring.iter().position(|stop| *stop == self.stop()).unwrap_or(0);
        let to = if forward { (at + 1) % ring.len() } else { (at + ring.len() - 1) % ring.len() };
        self.go_to(ring[to]);
        DialogStep::Redraw
    }

    /// What Enter does where the keyboard is: on a box it goes on to the next
    /// (the second one saves); on a button it presses it.
    fn activate(&mut self) -> DialogStep {
        match self.stop() {
            Stop::Field(FieldId::First) => {
                self.go_to(Stop::Field(FieldId::Second));
                DialogStep::Redraw
            }
            Stop::Field(FieldId::Second) | Stop::Button(ButtonId::Save) => self.save(),
            Stop::Button(button) => self.press(button),
        }
    }

    fn press(&mut self, button: ButtonId) -> DialogStep {
        match button {
            ButtonId::Save => self.save(),
            ButtonId::Cancel | ButtonId::Keep => DialogStep::Cancel,
            ButtonId::Switch => self.confirmed(),
            _ => DialogStep::Nothing,
        }
    }

    /// What a yes to the question means.
    fn confirmed(&self) -> DialogStep {
        if self.kind == DialogKind::Keychain { DialogStep::UseKeychain } else { DialogStep::UseKeyFile }
    }

    /// Checks the two boxes against the rule and, when they pass, hands the
    /// passphrase over; otherwise says why not, in words that never quote it.
    fn save(&mut self) -> DialogStep {
        let same = self.first.same_as(&self.second);
        if let Some(reason) = model::passphrase_problem(self.first.chars(), same) {
            self.problem = Some(reason.to_string());
            if self.first.chars() < model::MIN_PASSPHRASE_CHARS {
                self.go_to(Stop::Field(FieldId::First));
            } else {
                self.second.clear();
                self.go_to(Stop::Field(FieldId::Second));
            }
            return DialogStep::Redraw;
        }
        let secret = self.first.to_secret();
        if self.kind == DialogKind::Change { DialogStep::ChangePassphrase(secret) } else { DialogStep::UsePassphrase(secret) }
    }

    fn click(&mut self, area: Rect, col: u16, row: u16) -> DialogStep {
        let Some(placed) = place(area, &self.prompt()) else {
            return DialogStep::Nothing;
        };
        if let Some(button) = placed.button_at(col, row) {
            self.go_to(Stop::Button(button));
            return self.press(button);
        }
        if let Some(field) = placed.field_at(col, row) {
            self.go_to(Stop::Field(field));
            return DialogStep::Redraw;
        }
        DialogStep::Nothing
    }
}

fn is_escape(key: &KeyEvent) -> bool {
    let held = key.modifiers;
    !held.ctrl && !held.alt && !held.meta && key.key == LogicalKey::Named(NamedKey::Escape)
}

#[cfg(test)]
include!("dialog_input.tests.rs");
