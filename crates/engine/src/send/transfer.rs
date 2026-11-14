//! How the body travels: as it is when it is plain ASCII, 8 bit when the
//! server has said it takes that, quoted-printable (RFC 2045) otherwise.
//! Quoted-printable is always allowed, so it is also the answer when a line
//! is too long for the other two.

/// The longest line the other encodings allow, in bytes, without the CRLF.
const MAX_RAW_LINE: usize = 998;

/// The longest line of quoted-printable text, with its `=` at the end.
const QP_LINE: usize = 76;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    SevenBit,
    EightBit,
    QuotedPrintable,
}

impl Encoding {
    /// The value of `Content-Transfer-Encoding`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::SevenBit => "7bit",
            Self::EightBit => "8bit",
            Self::QuotedPrintable => "quoted-printable",
        }
    }
}

/// The encoding for a body (CRLF line ends). `eight_bit_ok` is whether the
/// server offered 8BITMIME.
pub fn choose(body: &str, eight_bit_ok: bool) -> Encoding {
    let lines_fit = body.split("\r\n").all(|line| line.len() <= MAX_RAW_LINE);
    if !lines_fit {
        Encoding::QuotedPrintable
    } else if body.is_ascii() {
        Encoding::SevenBit
    } else if eight_bit_ok {
        Encoding::EightBit
    } else {
        Encoding::QuotedPrintable
    }
}

/// The body's bytes as they go on the wire for `encoding`.
pub fn encode(body: &str, encoding: Encoding) -> Vec<u8> {
    match encoding {
        Encoding::SevenBit | Encoding::EightBit => body.as_bytes().to_vec(),
        Encoding::QuotedPrintable => quoted_printable(body.as_bytes()).into_bytes(),
    }
}

/// Quoted-printable text of `data`, whose line ends are CRLF. Lines are cut
/// with `=` at 76 characters, and a blank at the end of a line is spelled
/// out so it survives transport.
pub fn quoted_printable(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() + data.len() / 8);
    let mut line_len = 0;
    let mut at = 0;
    while at < data.len() {
        let byte = data[at];
        if byte == b'\r' && data.get(at + 1) == Some(&b'\n') {
            out.push_str("\r\n");
            line_len = 0;
            at += 2;
            continue;
        }
        let at_line_end = matches!(data.get(at + 1), None | Some(b'\r'));
        let unit = unit_of(byte, at_line_end);
        if line_len + unit.len() > QP_LINE - 1 {
            out.push_str("=\r\n");
            line_len = 0;
        }
        out.push_str(&unit);
        line_len += unit.len();
        at += 1;
    }
    out
}

/// One byte as quoted-printable text: itself, or `=XX`.
fn unit_of(byte: u8, at_line_end: bool) -> String {
    let plain = match byte {
        b'=' => false,
        b' ' | b'\t' => !at_line_end,
        33..=126 => true,
        _ => false,
    };
    if plain { char::from(byte).to_string() } else { format!("={byte:02X}") }
}

#[cfg(test)]
include!("transfer.tests.rs");
