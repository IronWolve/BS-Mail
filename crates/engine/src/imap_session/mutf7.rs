//! Mailbox names in modified UTF-7 (RFC 3501, section 5.1.3), decoded for
//! display. Commands keep using the name exactly as the server sent it.

/// The name as people read it. Malformed runs are left as they came.
pub fn decode(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut chars = name.chars();
    while let Some(c) = chars.next() {
        if c != '&' {
            out.push(c);
            continue;
        }
        let mut run = String::new();
        let mut closed = false;
        for d in chars.by_ref() {
            if d == '-' {
                closed = true;
                break;
            }
            run.push(d);
        }
        if !closed {
            out.push('&');
            out.push_str(&run);
        } else if run.is_empty() {
            out.push('&');
        } else {
            match decode_run(&run) {
                Some(text) => out.push_str(&text),
                None => {
                    out.push('&');
                    out.push_str(&run);
                    out.push('-');
                }
            }
        }
    }
    out
}

fn sextet(c: char) -> Option<u32> {
    match c {
        'A'..='Z' => Some(c as u32 - 'A' as u32),
        'a'..='z' => Some(c as u32 - 'a' as u32 + 26),
        '0'..='9' => Some(c as u32 - '0' as u32 + 52),
        '+' => Some(62),
        ',' => Some(63),
        _ => None,
    }
}

/// One base64 run (with `,` for `/`, no padding) holding UTF-16BE text.
fn decode_run(run: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(run.len() * 3 / 4);
    let (mut bits, mut count) = (0u32, 0u32);
    for c in run.chars() {
        bits = (bits << 6) | sextet(c)?;
        count += 6;
        if count >= 8 {
            count -= 8;
            bytes.push(((bits >> count) & 0xff) as u8);
        }
    }
    if bytes.len() % 2 != 0 {
        return None;
    }
    let units = bytes.chunks_exact(2).map(|pair| u16::from_be_bytes([pair[0], pair[1]]));
    char::decode_utf16(units).collect::<Result<String, _>>().ok()
}

#[cfg(test)]
include!("mutf7.tests.rs");
