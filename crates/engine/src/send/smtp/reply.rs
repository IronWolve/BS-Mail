//! A server's answer: a three-digit code and one or more lines of text.

/// One complete answer, possibly spread over several lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    pub code: u16,
    /// The text of each line, without the code and the separator.
    pub lines: Vec<String>,
}

impl Reply {
    /// 2xx: done.
    pub fn is_positive(&self) -> bool {
        self.code / 100 == 2
    }

    /// The text on one line, for a reason sentence.
    pub fn text(&self) -> String {
        self.lines.join(" ")
    }
}

/// One line of an answer, cut apart.
#[derive(Debug, PartialEq, Eq)]
pub struct ReplyLine<'a> {
    pub code: u16,
    /// A `-` after the code: more lines follow.
    pub more: bool,
    pub text: &'a str,
}

/// Cuts `NNN-text` or `NNN text` (or a bare `NNN`) apart; `None` for
/// anything else.
pub fn parse_line(line: &str) -> Option<ReplyLine<'_>> {
    let digits = line.get(..3)?;
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let code: u16 = digits.parse().ok()?;
    if !(200..600).contains(&code) {
        return None;
    }
    match line.as_bytes().get(3) {
        None => Some(ReplyLine { code, more: false, text: "" }),
        Some(b' ') => Some(ReplyLine { code, more: false, text: &line[4..] }),
        Some(b'-') => Some(ReplyLine { code, more: true, text: &line[4..] }),
        Some(_) => None,
    }
}

#[cfg(test)]
include!("reply.tests.rs");
