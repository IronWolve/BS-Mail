//! Cuts a query into tokens: words, quoted phrases, `field:value` pairs,
//! parentheses, the words `or` and `not`, quick filters such as `:unread`, and
//! a minus that negates the term right after it.

use super::error::SearchError;

/// The longest query accepted, in characters.
pub const MAX_QUERY_CHARS: usize = 2000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Open,
    Close,
    /// The word `or`, in any letter case.
    Or,
    /// The word `not`, or a minus in front of a term.
    Not,
    /// Plain or quoted text. Quoted text is never taken for `or` or `not`.
    Word(String),
    /// `name:value`, with the name in lower case.
    Field { name: String, value: String },
    /// A quick filter: a colon and a name, as in `:unread`, with the name in
    /// lower case. Only an unquoted word makes one.
    Quick(String),
}

/// The tokens of `text`, or the problem that stops it from being read.
pub fn lex(text: &str) -> Result<Vec<Token>, SearchError> {
    if text.chars().count() > MAX_QUERY_CHARS {
        return Err(SearchError::new(format!(
            "the search is longer than {MAX_QUERY_CHARS} characters"
        )));
    }
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut at = 0;
    while at < chars.len() {
        let c = chars[at];
        if c.is_whitespace() {
            at += 1;
        } else if c == '(' || c == ')' {
            out.push(if c == '(' { Token::Open } else { Token::Close });
            at += 1;
        } else if c == '"' {
            let (word, next) = read_quoted(&chars, at)?;
            out.push(Token::Word(word));
            at = next;
        } else if c == '-' {
            let after = chars.get(at + 1).copied();
            if after.is_none_or(|n| n.is_whitespace() || n == ')') {
                return Err(SearchError::new(
                    "a minus sign must come right before the term it negates, as in -from:alex",
                ));
            }
            out.push(Token::Not);
            at += 1;
        } else {
            let (token, next) = read_word(&chars, at)?;
            out.push(token);
            at = next;
        }
    }
    Ok(out)
}

/// A quoted string that starts at `start` (the opening quote). Inside, a
/// backslash before a quote or another backslash takes it literally.
fn read_quoted(chars: &[char], start: usize) -> Result<(String, usize), SearchError> {
    let mut text = String::new();
    let mut at = start + 1;
    while at < chars.len() {
        match chars[at] {
            '"' => return Ok((text, at + 1)),
            '\\' if matches!(chars.get(at + 1), Some('"' | '\\')) => {
                text.push(chars[at + 1]);
                at += 2;
            }
            other => {
                text.push(other);
                at += 1;
            }
        }
    }
    Err(SearchError::new(format!(
        "the quote that starts at character {} is never closed",
        start + 1
    )))
}

/// A word, a `name:value` pair or `name:"quoted value"`, from `start`.
fn read_word(chars: &[char], start: usize) -> Result<(Token, usize), SearchError> {
    let mut word = String::new();
    let mut at = start;
    while let Some(&c) = chars.get(at) {
        if c.is_whitespace() || c == '(' || c == ')' {
            break;
        }
        if c == '"' {
            let Some(name) = word.strip_suffix(':').filter(|n| is_field_name(n)) else {
                break;
            };
            let (value, next) = read_quoted(chars, at)?;
            let name = name.to_ascii_lowercase();
            return Ok((Token::Field { name, value }, next));
        }
        word.push(c);
        at += 1;
    }
    Ok((classify(word), at))
}

fn classify(word: String) -> Token {
    if let Some(name) = word.strip_prefix(':')
        && is_field_name(name)
    {
        return Token::Quick(name.to_ascii_lowercase());
    }
    if let Some((name, value)) = word.split_once(':')
        && is_field_name(name)
    {
        return Token::Field { name: name.to_ascii_lowercase(), value: value.to_owned() };
    }
    if word.eq_ignore_ascii_case("or") {
        Token::Or
    } else if word.eq_ignore_ascii_case("not") {
        Token::Not
    } else {
        Token::Word(word)
    }
}

/// A field name is made of letters only, so `10:30` stays a plain word.
fn is_field_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphabetic())
}

#[cfg(test)]
include!("lexer.tests.rs");
