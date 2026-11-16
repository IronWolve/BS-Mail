//! The text of a search as it goes to the server: pieces of plain command
//! text with strings that need it carried as literals.
//!
//! A string goes inside quotes when every character is printable ASCII and
//! none is a brace, with the backslash and the quote escaped. Anything else
//! (accents, other scripts, line breaks, tabs and other control characters,
//! braces) is carried as a literal: the length in braces, a line end, and the
//! bytes only after the server says it is ready. That way no text can break
//! out of its quoting or turn into a second command.

/// One piece of a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    /// Command text with no line break in it.
    Text(String),
    /// The bytes of a literal, as text.
    Literal(String),
}

/// The search keys of one query, ready to follow `UID SEARCH`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keys {
    parts: Vec<Part>,
    needs_charset: bool,
}

impl Keys {
    /// The key that matches messages whose header `name` holds `value`
    /// (`HEADER name "value"`), for looking a message up by its Message-ID.
    /// `None` when the value is not plain printable ASCII, since it is then
    /// not worth a literal.
    pub fn header_holds(name: &str, value: &str) -> Option<Keys> {
        let plain = |text: &str| !text.is_empty() && text.chars().all(|c| c.is_ascii_graphic());
        if !plain(name) || !plain(value) {
            return None;
        }
        let mut writer = KeyWriter::default();
        writer.text(&format!("HEADER {name} "));
        writer.string(value);
        writer.finish().ok()
    }

    /// The keys that match every message, `ALL`.
    pub fn all() -> Keys {
        Keys { parts: vec![Part::Text("ALL".to_owned())], needs_charset: false }
    }

    /// The keys that match the messages not read yet, `UNSEEN`.
    pub fn unseen() -> Keys {
        Keys { parts: vec![Part::Text("UNSEEN".to_owned())], needs_charset: false }
    }

    /// The pieces, in order.
    pub fn parts(&self) -> &[Part] {
        &self.parts
    }

    /// True when some string is not plain ASCII, so the command must name
    /// the character set its strings are in.
    pub fn needs_charset(&self) -> bool {
        self.needs_charset
    }

    /// The keys as the server reads them once it has said it is ready for
    /// each literal: every literal shows as its length in braces, a line end
    /// and its bytes.
    #[cfg(test)]
    pub fn wire_text(&self) -> String {
        wire_text(&self.parts)
    }
}

/// `parts` as the server reads them, literals in place.
#[cfg(test)]
pub fn wire_text(parts: &[Part]) -> String {
    let mut out = String::new();
    for part in parts {
        match part {
            Part::Text(text) => out.push_str(text),
            Part::Literal(bytes) => {
                out.push_str(&format!("{{{}}}\r\n{bytes}", bytes.len()));
            }
        }
    }
    out
}

/// Builds the pieces of a command as text and strings are added.
#[derive(Debug, Default)]
pub(super) struct KeyWriter {
    parts: Vec<Part>,
    needs_charset: bool,
    problem: Option<String>,
}

impl KeyWriter {
    pub(super) fn text(&mut self, text: &str) {
        if let Some(Part::Text(last)) = self.parts.last_mut() {
            last.push_str(text);
        } else {
            self.parts.push(Part::Text(text.to_owned()));
        }
    }

    /// Adds a string the server will read as the text it is.
    pub(super) fn string(&mut self, value: &str) {
        let plain = value.chars().all(|c| (' '..='~').contains(&c) && c != '{' && c != '}');
        if plain {
            let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
            self.text(&format!("\"{escaped}\""));
        } else {
            self.needs_charset |= !value.is_ascii();
            self.parts.push(Part::Literal(value.to_owned()));
        }
    }

    /// Notes something that must not be sent; `finish` then refuses.
    pub(super) fn fail(&mut self, problem: &str) {
        self.problem.get_or_insert_with(|| problem.to_owned());
    }

    pub(super) fn finish(self) -> Result<Keys, String> {
        match self.problem {
            Some(problem) => Err(problem),
            None => Ok(Keys { parts: self.parts, needs_charset: self.needs_charset }),
        }
    }
}

#[cfg(test)]
include!("wire.tests.rs");
