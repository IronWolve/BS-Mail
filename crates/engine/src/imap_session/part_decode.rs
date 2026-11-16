//! Reading the transfer encoding of one attachment, a slice at a time, so a
//! large file is never held twice: the slices that arrive from the server are
//! decoded as they come and only the file itself is kept. Base64 and
//! quoted-printable are decoded; anything else (7bit, 8bit, binary, a name
//! nobody knows) is the file as it is. The decoders are forgiving of what
//! real mailers write (line breaks and stray blanks inside base64, an `=`
//! that is no escape in quoted-printable) and never panic or loop.

use async_imap::imap_proto::ContentEncoding;

/// How the bytes of a part were encoded for transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transfer {
    Identity,
    Base64,
    QuotedPrintable,
}

impl Transfer {
    pub fn of(encoding: &ContentEncoding<'_>) -> Self {
        match encoding {
            ContentEncoding::Base64 => Self::Base64,
            ContentEncoding::QuotedPrintable => Self::QuotedPrintable,
            ContentEncoding::Other(name) if name.eq_ignore_ascii_case("base64") => Self::Base64,
            ContentEncoding::Other(name) if name.eq_ignore_ascii_case("quoted-printable") => Self::QuotedPrintable,
            _ => Self::Identity,
        }
    }
}

/// Where the quoted-printable reader is inside an escape or a soft break.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Qp {
    Plain,
    /// Just read `=`.
    Escape,
    /// Read `=` and one hex digit.
    Escape1(u8),
    /// Read `=` and blanks: the end of a soft line break is awaited.
    SoftBlanks,
    /// Read `=`, blanks and a CR: the LF is awaited.
    SoftCr,
}

/// Decodes the slices of one part, in order, into the file.
pub struct PartDecoder {
    transfer: Transfer,
    out: Vec<u8>,
    /// Base64: the bits read so far and how many 6 bit groups they hold.
    bits: u32,
    groups: u8,
    finished: bool,
    /// Quoted-printable: the escape state, blanks waiting to learn whether
    /// they end a line (and so are dropped), and a CR waiting for its LF.
    qp: Qp,
    blanks: Vec<u8>,
    cr: bool,
}

impl PartDecoder {
    pub fn new(transfer: Transfer) -> Self {
        Self { transfer, out: Vec::new(), bits: 0, groups: 0, finished: false, qp: Qp::Plain, blanks: Vec::new(), cr: false }
    }

    /// Decodes the next slice.
    pub fn push(&mut self, slice: &[u8]) {
        match self.transfer {
            Transfer::Identity => self.out.extend_from_slice(slice),
            Transfer::Base64 => slice.iter().for_each(|byte| self.base64(*byte)),
            Transfer::QuotedPrintable => slice.iter().for_each(|byte| self.quoted_printable(*byte)),
        }
    }

    /// The decoded file, once the last slice was pushed.
    pub fn finish(mut self) -> Vec<u8> {
        match self.transfer {
            Transfer::Identity => {}
            Transfer::Base64 => self.flush_base64(),
            Transfer::QuotedPrintable => self.flush_quoted_printable(),
        }
        self.out
    }

    fn base64(&mut self, byte: u8) {
        if self.finished {
            return;
        }
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => {
                self.flush_base64();
                self.finished = true;
                return;
            }
            // Line breaks, blanks and anything else that is no digit.
            _ => return,
        };
        self.bits = (self.bits << 6) | u32::from(value);
        self.groups += 1;
        if self.groups == 4 {
            self.out.extend_from_slice(&[(self.bits >> 16) as u8, (self.bits >> 8) as u8, self.bits as u8]);
            self.bits = 0;
            self.groups = 0;
        }
    }

    /// The bytes of a last group of two or three digits.
    fn flush_base64(&mut self) {
        match self.groups {
            2 => self.out.push((self.bits >> 4) as u8),
            3 => self.out.extend_from_slice(&[(self.bits >> 10) as u8, (self.bits >> 2) as u8]),
            _ => {}
        }
        self.bits = 0;
        self.groups = 0;
    }

    fn quoted_printable(&mut self, byte: u8) {
        match self.qp {
            Qp::Plain => self.qp_plain(byte),
            Qp::Escape => match (hex(byte), byte) {
                (Some(high), _) => self.qp = Qp::Escape1(high),
                (None, b' ' | b'\t') => self.qp = Qp::SoftBlanks,
                (None, b'\r') => self.qp = Qp::SoftCr,
                (None, b'\n') => self.qp = Qp::Plain,
                // An `=` that begins nothing is a plain `=`.
                (None, _) => {
                    self.emit(b'=');
                    self.qp = Qp::Plain;
                    self.qp_plain(byte);
                }
            },
            Qp::Escape1(high) => match hex(byte) {
                Some(low) => {
                    self.emit((high << 4) | low);
                    self.qp = Qp::Plain;
                }
                None => {
                    self.emit(b'=');
                    self.emit(HEX_DIGITS[usize::from(high)]);
                    self.qp = Qp::Plain;
                    self.qp_plain(byte);
                }
            },
            Qp::SoftBlanks => match byte {
                b' ' | b'\t' => {}
                b'\r' => self.qp = Qp::SoftCr,
                b'\n' => self.qp = Qp::Plain,
                _ => {
                    self.qp = Qp::Plain;
                    self.qp_plain(byte);
                }
            },
            Qp::SoftCr => {
                self.qp = Qp::Plain;
                if byte != b'\n' {
                    self.qp_plain(byte);
                }
            }
        }
    }

    fn qp_plain(&mut self, byte: u8) {
        if self.cr && byte != b'\n' {
            self.cr = false;
            self.out.push(b'\r');
        }
        match byte {
            b'=' => self.qp = Qp::Escape,
            b' ' | b'\t' => self.blanks.push(byte),
            b'\r' => self.cr = true,
            b'\n' => {
                // Blanks before a line break are transport padding.
                self.blanks.clear();
                if std::mem::take(&mut self.cr) {
                    self.out.push(b'\r');
                }
                self.out.push(b'\n');
            }
            other => self.emit(other),
        }
    }

    /// A decoded byte after any blanks that were waiting for what follows.
    fn emit(&mut self, byte: u8) {
        self.out.append(&mut self.blanks);
        self.out.push(byte);
    }

    fn flush_quoted_printable(&mut self) {
        match self.qp {
            Qp::Escape => self.emit(b'='),
            Qp::Escape1(high) => {
                self.emit(b'=');
                self.emit(HEX_DIGITS[usize::from(high)]);
            }
            Qp::Plain | Qp::SoftBlanks | Qp::SoftCr => {}
        }
        self.out.append(&mut self.blanks);
        if self.cr {
            self.out.push(b'\r');
        }
        self.qp = Qp::Plain;
        self.cr = false;
    }
}

const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
include!("part_decode.tests.rs");
