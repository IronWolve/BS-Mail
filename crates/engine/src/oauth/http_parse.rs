//! Reading an HTTP/1.1 reply from bytes: the status line and headers, then
//! the body by `Content-Length`, by chunks, or until the connection closes.
//! No network here, so every rule can be tested with plain byte strings.

/// The most header bytes read before the reply is called unreadable.
pub const MAX_HEAD: usize = 16 * 1024;

/// How the body is delimited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framing {
    Length(usize),
    Chunked,
    UntilClose,
}

/// The status and how to read the rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Head {
    pub status: u16,
    pub framing: Framing,
    /// Bytes the status line and headers took, including the blank line.
    pub length: usize,
}

/// The body so far. Its debug output shows only the size: it may hold tokens.
#[derive(PartialEq, Eq)]
pub enum Body {
    Complete(Vec<u8>),
    More,
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Complete(bytes) => write!(f, "Complete(<{} bytes>)", bytes.len()),
            Self::More => f.write_str("More"),
        }
    }
}

/// Reads the head from the start of `buf`; `None` until the blank line has
/// arrived.
pub fn parse_head(buf: &[u8]) -> Result<Option<Head>, String> {
    let Some(end) = buf.windows(4).position(|w| w == b"\r\n\r\n") else {
        return Ok(None);
    };
    let text = std::str::from_utf8(&buf[..end]).map_err(|_| "the reply's headers are not text".to_owned())?;
    let mut lines = text.split("\r\n");
    let status = status_of(lines.next().unwrap_or(""))?;
    let (mut length, mut chunked) = (None::<usize>, false);
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            return Err("a header line has no colon".to_owned());
        };
        let value = value.trim();
        if name.eq_ignore_ascii_case("content-length") {
            let parsed = parse_length(value)?;
            if length.is_some_and(|earlier| earlier != parsed) {
                return Err("the reply gives two different lengths".to_owned());
            }
            length = Some(parsed);
        } else if name.eq_ignore_ascii_case("transfer-encoding") {
            chunked = value.rsplit(',').next().is_some_and(|last| last.trim().eq_ignore_ascii_case("chunked"));
        }
    }
    let framing = match (chunked, length) {
        (true, _) => Framing::Chunked,
        (false, Some(n)) => Framing::Length(n),
        (false, None) if status == 204 => Framing::Length(0),
        (false, None) => Framing::UntilClose,
    };
    Ok(Some(Head { status, framing, length: end + 4 }))
}

fn status_of(line: &str) -> Result<u16, String> {
    let mut parts = line.split(' ');
    let version = parts.next().unwrap_or("");
    if version != "HTTP/1.1" && version != "HTTP/1.0" {
        return Err("the reply is not HTTP".to_owned());
    }
    let code = parts.next().unwrap_or("");
    if code.len() != 3 || !code.bytes().all(|b| b.is_ascii_digit()) {
        return Err("the reply has no status code".to_owned());
    }
    code.parse().map_err(|_| "the reply has no status code".to_owned())
}

fn parse_length(value: &str) -> Result<usize, String> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err("the reply's length is not a number".to_owned());
    }
    value.parse().map_err(|_| "the reply's length is too large".to_owned())
}

/// The body in `data` (everything after the head). `closed` says the server
/// has closed its end; `cap` is the most body bytes allowed.
pub fn read_body(framing: Framing, data: &[u8], closed: bool, cap: usize) -> Result<Body, String> {
    match framing {
        Framing::Length(n) if n > cap => Err(too_large()),
        Framing::Length(n) if data.len() >= n => Ok(Body::Complete(data[..n].to_vec())),
        Framing::Length(_) if closed => Err("the reply ended before its whole body arrived".to_owned()),
        Framing::UntilClose if data.len() > cap => Err(too_large()),
        Framing::UntilClose if closed => Ok(Body::Complete(data.to_vec())),
        Framing::Length(_) | Framing::UntilClose => Ok(Body::More),
        Framing::Chunked => read_chunks(data, closed, cap),
    }
}

/// The text the caller shows when a body goes over the cap.
pub fn too_large() -> String {
    "the reply is larger than this program reads".to_owned()
}

fn read_chunks(data: &[u8], closed: bool, cap: usize) -> Result<Body, String> {
    let (mut at, mut body) = (0usize, Vec::new());
    loop {
        let Some(line_end) = find_line(&data[at..]) else {
            return incomplete(closed);
        };
        let line = std::str::from_utf8(&data[at..at + line_end]).map_err(|_| "a chunk size is not text".to_owned())?;
        let size_text = line.split(';').next().unwrap_or("").trim();
        if size_text.is_empty() || !size_text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("a chunk size is not a number".to_owned());
        }
        let size = usize::from_str_radix(size_text, 16).map_err(|_| "a chunk is too large".to_owned())?;
        at += line_end + 2;
        if size == 0 {
            return trailers_end(&data[at..], closed).map(|done| if done { Body::Complete(body) } else { Body::More });
        }
        if body.len().saturating_add(size) > cap {
            return Err(too_large());
        }
        if data.len() < at + size + 2 {
            return incomplete(closed);
        }
        if &data[at + size..at + size + 2] != b"\r\n" {
            return Err("a chunk is not followed by a line end".to_owned());
        }
        body.extend_from_slice(&data[at..at + size]);
        at += size + 2;
    }
}

/// True when the trailer section (possibly empty) has fully arrived.
fn trailers_end(rest: &[u8], closed: bool) -> Result<bool, String> {
    if rest.starts_with(b"\r\n") || rest.windows(4).any(|w| w == b"\r\n\r\n") {
        return Ok(true);
    }
    if closed { Err("the reply ended inside its last chunk".to_owned()) } else { Ok(false) }
}

fn find_line(data: &[u8]) -> Option<usize> {
    data.windows(2).position(|w| w == b"\r\n")
}

fn incomplete(closed: bool) -> Result<Body, String> {
    if closed { Err("the reply ended inside a chunk".to_owned()) } else { Ok(Body::More) }
}

#[cfg(test)]
include!("http_parse.tests.rs");
