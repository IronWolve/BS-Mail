//! The message as the DATA command carries it: dot-stuffed and closed with a
//! line holding one dot.

/// True when every line ends with CRLF and no bare CR or LF is inside the
/// message. A bare line break could end the DATA early and let what follows
/// be read as commands.
pub fn is_wire_safe(message: &[u8]) -> bool {
    message.iter().enumerate().all(|(at, byte)| match byte {
        b'\n' => at > 0 && message[at - 1] == b'\r',
        b'\r' => message.get(at + 1) == Some(&b'\n'),
        _ => true,
    })
}

/// The bytes to write after the server's 354: the message with a dot added
/// in front of every line that starts with one, a final CRLF if it lacked
/// one, and the closing `.` line.
pub fn dot_stuffed(message: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(message.len() + message.len() / 64 + 8);
    let mut line_start = true;
    for &byte in message {
        if line_start && byte == b'.' {
            out.push(b'.');
        }
        out.push(byte);
        line_start = byte == b'\n';
    }
    if !out.ends_with(b"\r\n") {
        out.extend_from_slice(b"\r\n");
    }
    out.extend_from_slice(b".\r\n");
    out
}

#[cfg(test)]
include!("data.tests.rs");
