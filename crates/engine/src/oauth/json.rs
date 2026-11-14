//! A small, strict JSON reader for the provider's replies. It reads one
//! complete value, refuses anything after it, and never goes deeper than
//! `MAX_DEPTH` levels. The replies are small objects, so this keeps only what
//! the sign-in needs: strings, numbers, and the shape around them.

const MAX_DEPTH: usize = 16;

/// A parsed JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    List(Vec<Value>),
    /// Members in the order written; when a name repeats, the last wins.
    Object(Vec<(String, Value)>),
}

impl Value {
    /// The member of an object with this name.
    pub fn get(&self, name: &str) -> Option<&Value> {
        match self {
            Self::Object(members) => members.iter().rev().find(|(n, _)| n == name).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    /// A whole number of seconds from a JSON number or a numeric string.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn as_seconds(&self) -> Option<u64> {
        let number = match self {
            Self::Number(n) => *n,
            Self::Text(text) => text.trim().parse::<f64>().ok()?,
            _ => return None,
        };
        (number.is_finite() && (0.0..=1.0e12).contains(&number)).then(|| number.trunc() as u64)
    }
}

/// Reads `text` as one JSON value.
pub fn parse(text: &str) -> Result<Value, String> {
    let mut reader = Reader { bytes: text.as_bytes(), at: 0 };
    reader.skip_blanks();
    let value = reader.value(0)?;
    reader.skip_blanks();
    if reader.at != reader.bytes.len() {
        return Err(reader.fail("more text after the value"));
    }
    Ok(value)
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn fail(&self, what: &str) -> String {
        format!("not valid JSON at byte {}: {what}", self.at)
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn skip_blanks(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.at += 1;
        }
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        if self.peek() == Some(byte) {
            self.at += 1;
            Ok(())
        } else {
            Err(self.fail(&format!("expected `{}`", char::from(byte))))
        }
    }

    fn word(&mut self, word: &str, value: Value) -> Result<Value, String> {
        if self.bytes[self.at..].starts_with(word.as_bytes()) {
            self.at += word.len();
            Ok(value)
        } else {
            Err(self.fail("unknown word"))
        }
    }

    fn value(&mut self, depth: usize) -> Result<Value, String> {
        if depth > MAX_DEPTH {
            return Err(self.fail("nested too deeply"));
        }
        match self.peek() {
            Some(b'{') => self.object(depth),
            Some(b'[') => self.list(depth),
            Some(b'"') => self.text().map(Value::Text),
            Some(b't') => self.word("true", Value::Bool(true)),
            Some(b'f') => self.word("false", Value::Bool(false)),
            Some(b'n') => self.word("null", Value::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.fail("a value was expected")),
        }
    }

    fn object(&mut self, depth: usize) -> Result<Value, String> {
        self.expect(b'{')?;
        let mut members = Vec::new();
        self.skip_blanks();
        if self.peek() == Some(b'}') {
            self.at += 1;
            return Ok(Value::Object(members));
        }
        loop {
            self.skip_blanks();
            let name = self.text()?;
            self.skip_blanks();
            self.expect(b':')?;
            self.skip_blanks();
            members.push((name, self.value(depth + 1)?));
            self.skip_blanks();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b'}') => {
                    self.at += 1;
                    return Ok(Value::Object(members));
                }
                _ => return Err(self.fail("expected `,` or `}`")),
            }
        }
    }

    fn list(&mut self, depth: usize) -> Result<Value, String> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_blanks();
        if self.peek() == Some(b']') {
            self.at += 1;
            return Ok(Value::List(items));
        }
        loop {
            self.skip_blanks();
            items.push(self.value(depth + 1)?);
            self.skip_blanks();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b']') => {
                    self.at += 1;
                    return Ok(Value::List(items));
                }
                _ => return Err(self.fail("expected `,` or `]`")),
            }
        }
    }

    fn number(&mut self) -> Result<Value, String> {
        let start = self.at;
        if self.peek() == Some(b'-') {
            self.at += 1;
        }
        let digits = |reader: &mut Self| {
            let before = reader.at;
            while matches!(reader.peek(), Some(b'0'..=b'9')) {
                reader.at += 1;
            }
            reader.at > before
        };
        if !digits(self) {
            return Err(self.fail("digits were expected"));
        }
        if self.peek() == Some(b'.') {
            self.at += 1;
            if !digits(self) {
                return Err(self.fail("digits were expected after the point"));
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.at += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.at += 1;
            }
            if !digits(self) {
                return Err(self.fail("digits were expected in the exponent"));
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.at]).map_err(|_| self.fail("bad number"))?;
        text.parse::<f64>().map(Value::Number).map_err(|_| self.fail("bad number"))
    }

    fn text(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let Some(byte) = self.peek() else {
                return Err(self.fail("the string does not end"));
            };
            self.at += 1;
            match byte {
                b'"' => break,
                b'\\' => self.escape(&mut out)?,
                0..=0x1f => return Err(self.fail("a raw control character in a string")),
                other => out.push(other),
            }
        }
        String::from_utf8(out).map_err(|_| self.fail("the string is not valid UTF-8"))
    }

    fn escape(&mut self, out: &mut Vec<u8>) -> Result<(), String> {
        let Some(code) = self.peek() else {
            return Err(self.fail("the string does not end"));
        };
        self.at += 1;
        let plain = match code {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{8}',
            b'f' => '\u{c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => self.unicode()?,
            _ => return Err(self.fail("an unknown escape")),
        };
        let mut buffer = [0u8; 4];
        out.extend_from_slice(plain.encode_utf8(&mut buffer).as_bytes());
        Ok(())
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let digits = self.bytes.get(self.at..self.at + 4).ok_or_else(|| self.fail("a short \\u escape"))?;
        let text = std::str::from_utf8(digits).map_err(|_| self.fail("a bad \\u escape"))?;
        let value = u32::from_str_radix(text, 16).map_err(|_| self.fail("a bad \\u escape"))?;
        self.at += 4;
        Ok(value)
    }

    /// The character of a `\u` escape, joining a surrogate pair.
    fn unicode(&mut self) -> Result<char, String> {
        let first = self.hex4()?;
        if !(0xD800..0xDC00).contains(&first) {
            return char::from_u32(first).ok_or_else(|| self.fail("a lone surrogate"));
        }
        if self.bytes[self.at..].starts_with(b"\\u") {
            self.at += 2;
            let second = self.hex4()?;
            if (0xDC00..0xE000).contains(&second) {
                let joined = 0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00);
                return char::from_u32(joined).ok_or_else(|| self.fail("a bad surrogate pair"));
            }
        }
        Err(self.fail("a lone surrogate"))
    }
}

#[cfg(test)]
include!("json.tests.rs");
