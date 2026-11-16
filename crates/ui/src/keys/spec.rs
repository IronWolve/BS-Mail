//! One key as the `[keys]` table writes it, and how it is told from a key
//! press: `r`, `R`, `F5`, `Ctrl+k`, `Alt+x`, `Shift+F6`, `PageUp`.
//!
//! A letter is written as the letter itself: `R` is Shift and `r`, and
//! `Shift+r` means the same. With Ctrl or Alt a letter is written in lower
//! case, and an upper case one means Shift is held as well. A key that needs
//! Shift to be typed (`!`, `?`, `*`) is written as that character; `Shift+!`
//! is refused, since which key makes it depends on the keyboard.

use surface::{KeyEvent, LogicalKey, NamedKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeySpec {
    pub key: LogicalKey,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl KeySpec {
    /// A character with no modifier.
    pub const fn ch(ch: char) -> Self {
        Self { key: LogicalKey::Char(ch), ctrl: false, alt: false, shift: false }
    }

    /// A named key with no modifier.
    pub const fn named(key: NamedKey) -> Self {
        Self { key: LogicalKey::Named(key), ctrl: false, alt: false, shift: false }
    }

    /// The same key with Shift held.
    pub const fn shifted(mut self) -> Self {
        self.shift = true;
        self
    }

    /// The same key with Ctrl held.
    pub const fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    /// Reads the text of one key, or says in plain words why it is not one.
    pub fn parse(text: &str) -> Result<Self, String> {
        let text = text.trim();
        if text.is_empty() {
            return Err("no key was given".to_string());
        }
        let (modifiers, key) = split(text);
        let mut spec = Self { key: LogicalKey::Char(' '), ctrl: false, alt: false, shift: false };
        for word in modifiers.split('+').map(str::trim).filter(|word| !word.is_empty()) {
            match word.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => spec.ctrl = true,
                "alt" => spec.alt = true,
                "shift" => spec.shift = true,
                _ => return Err(format!("`{word}` is not a modifier: use Ctrl, Alt or Shift")),
            }
        }
        spec.key = key_of(key)?;
        spec.settle(text)
    }

    /// The spelling every key has one of: letters are as typed, Shift folded
    /// into an upper case letter, and a refusal for what cannot be said.
    fn settle(mut self, text: &str) -> Result<Self, String> {
        let LogicalKey::Char(ch) = self.key else {
            return Ok(self);
        };
        let held = self.ctrl || self.alt;
        match (ch.is_alphabetic(), held) {
            (true, false) => {
                if self.shift {
                    self.key = LogicalKey::Char(upper(ch));
                    self.shift = false;
                }
            }
            (true, true) => {
                // An upper case letter with Ctrl or Alt says Shift too.
                if ch.is_uppercase() {
                    self.shift = true;
                }
                self.key = LogicalKey::Char(lower(ch));
            }
            (false, _) if self.shift && ch != ' ' => {
                return Err(format!("`{text}`: write the character itself, as it is typed, without Shift+"));
            }
            _ => {}
        }
        Ok(self)
    }

    /// The key as the table writes it: `Ctrl+k`, `F5`, `R`, `Space`.
    pub fn text(&self) -> String {
        let mut out = String::new();
        for (held, word) in [(self.ctrl, "Ctrl+"), (self.alt, "Alt+"), (self.shift, "Shift+")] {
            if held {
                out.push_str(word);
            }
        }
        match self.key {
            LogicalKey::Char(' ') => out.push_str("Space"),
            LogicalKey::Char(ch) => out.push(ch),
            LogicalKey::Named(named) => out.push_str(&named_text(named)),
        }
        out
    }

    /// The text of the keycap in the key bar: a letter in capitals, as the
    /// picture draws them (`R`), and a capital as Shift and the letter.
    pub fn cap_text(&self) -> String {
        match self.key {
            LogicalKey::Char(ch) if ch.is_lowercase() && !self.ctrl && !self.alt => upper(ch).to_string(),
            LogicalKey::Char(ch) if ch.is_uppercase() && !self.ctrl && !self.alt => format!("Shift+{ch}"),
            _ => self.text(),
        }
    }

    /// Whether a key press is this key. Shift is only asked for where it
    /// tells keys apart: a lower case letter is not matched with it held, a
    /// named key is matched with exactly the Shift of the spec, and any other
    /// character (an upper case letter, `?`, `*`) is matched however Shift is
    /// reported, since the layout makes it.
    pub fn matches(&self, event: &KeyEvent) -> bool {
        let held = event.modifiers;
        if held.meta || held.ctrl != self.ctrl || held.alt != self.alt {
            return false;
        }
        match (self.key, event.key) {
            (LogicalKey::Named(wanted), LogicalKey::Named(got)) => wanted == got && held.shift == self.shift,
            (LogicalKey::Char(wanted), LogicalKey::Char(got)) if self.ctrl || self.alt => {
                lower(wanted) == lower(got) && held.shift == self.shift
            }
            (LogicalKey::Char(wanted), LogicalKey::Char(got)) if wanted.is_lowercase() => wanted == got && !held.shift,
            (LogicalKey::Char(wanted), LogicalKey::Char(got)) => wanted == got,
            _ => false,
        }
    }
}

/// The modifiers part and the key part of `text`: everything before the last
/// `+` that is not the key itself.
fn split(text: &str) -> (&str, &str) {
    if text == "+" {
        return ("", "+");
    }
    if let Some(front) = text.strip_suffix("++") {
        return (front, "+");
    }
    text.rsplit_once('+').unwrap_or(("", text))
}

fn key_of(text: &str) -> Result<LogicalKey, String> {
    let text = text.trim();
    let mut chars = text.chars();
    if let (Some(only), None) = (chars.next(), chars.next())
        && !only.is_control()
        && !only.is_whitespace()
    {
        return Ok(LogicalKey::Char(only));
    }
    let lower = text.to_ascii_lowercase();
    let named = match lower.as_str() {
        "space" => return Ok(LogicalKey::Char(' ')),
        "enter" | "return" => NamedKey::Enter,
        "esc" | "escape" => NamedKey::Escape,
        "tab" => NamedKey::Tab,
        "backspace" => NamedKey::Backspace,
        "delete" | "del" => NamedKey::Delete,
        "insert" | "ins" => NamedKey::Insert,
        "home" => NamedKey::Home,
        "end" => NamedKey::End,
        "pageup" | "pgup" => NamedKey::PageUp,
        "pagedown" | "pgdn" => NamedKey::PageDown,
        "up" => NamedKey::Up,
        "down" => NamedKey::Down,
        "left" => NamedKey::Left,
        "right" => NamedKey::Right,
        "menu" => NamedKey::Menu,
        other => match other.strip_prefix('f').and_then(|digits| digits.parse::<u8>().ok()) {
            Some(number @ 1..=24) => NamedKey::Function(number),
            _ => return Err(format!("`{text}` is not a key: a character, or a name such as F5, Tab or PageUp")),
        },
    };
    Ok(LogicalKey::Named(named))
}

fn named_text(key: NamedKey) -> String {
    match key {
        NamedKey::Enter => "Enter".to_string(),
        NamedKey::Escape => "Esc".to_string(),
        NamedKey::Tab => "Tab".to_string(),
        NamedKey::Backspace => "Backspace".to_string(),
        NamedKey::Delete => "Delete".to_string(),
        NamedKey::Insert => "Insert".to_string(),
        NamedKey::Home => "Home".to_string(),
        NamedKey::End => "End".to_string(),
        NamedKey::PageUp => "PageUp".to_string(),
        NamedKey::PageDown => "PageDown".to_string(),
        NamedKey::Up => "Up".to_string(),
        NamedKey::Down => "Down".to_string(),
        NamedKey::Left => "Left".to_string(),
        NamedKey::Right => "Right".to_string(),
        NamedKey::Menu => "Menu".to_string(),
        NamedKey::Function(number) => format!("F{number}"),
    }
}

fn upper(ch: char) -> char {
    ch.to_uppercase().next().unwrap_or(ch)
}

fn lower(ch: char) -> char {
    ch.to_lowercase().next().unwrap_or(ch)
}

#[cfg(test)]
include!("spec.tests.rs");
