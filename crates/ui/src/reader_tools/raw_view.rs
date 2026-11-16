//! The raw headers or the whole source of a message, shown in the reading
//! pane in place of the message until Escape (or another message) brings it
//! back. The text comes from the engine already cleaned of control codes; it
//! is only ever drawn.

use model::{BodyRole, MessageId, RawPart, StyledLine, StyledSpan};

/// What the pane shows instead of the message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawView {
    pub message: MessageId,
    pub part: RawPart,
    /// The lines once they have arrived; `None` while they are awaited.
    lines: Option<Vec<StyledLine>>,
}

impl RawView {
    /// A view of `part` of `message` that is waiting for its text.
    pub fn waiting(message: MessageId, part: RawPart) -> Self {
        Self { message, part, lines: None }
    }

    /// The text arrived: every line of it, drawn as it is (no highlighting).
    pub fn arrived(&mut self, text: &str) {
        self.lines = Some(
            text.split('\n')
                .map(|line| StyledLine { spans: vec![StyledSpan { text: line.to_owned(), role: BodyRole::Text }] })
                .collect(),
        );
    }

    /// The lines to draw: the text, or none while it is awaited.
    pub fn lines(&self) -> &[StyledLine] {
        self.lines.as_deref().unwrap_or_default()
    }

    /// The line over the text that says what it is, and how to leave it.
    pub fn heading(&self) -> StyledLine {
        let what = match (self.part, self.lines.is_some()) {
            (RawPart::Headers, true) => "The headers of this message, as received. Esc goes back to the message.",
            (RawPart::Source, true) => "The source of this message, as received. Esc goes back to the message.",
            (RawPart::Headers, false) => "Reading the headers of this message...",
            (RawPart::Source, false) => "Reading the source of this message...",
        };
        StyledLine { spans: vec![StyledSpan { text: what.to_owned(), role: BodyRole::Lead }] }
    }
}

#[cfg(test)]
include!("raw_view.tests.rs");
