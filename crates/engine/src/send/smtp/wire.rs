//! The connection to the mail server: commands out, answers in, every step
//! under a time limit. Nothing here knows what the commands mean.

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use super::reply::{Reply, parse_line};
use crate::imap_session::MailStream;

/// The longest answer line accepted, in bytes. The standard allows 512.
const MAX_LINE: usize = 4096;

/// The most lines one answer may have.
const MAX_REPLY_LINES: usize = 200;

/// Why a read or write did not complete.
#[derive(Debug, PartialEq, Eq)]
pub enum WireError {
    /// The server hung up.
    Closed,
    /// The limit passed without an answer.
    TimedOut,
    /// The network failed.
    Io(String),
    /// The server said something that is not an answer.
    Unreadable(String),
}

/// A connection with whatever has been read and not yet used.
pub struct Wire {
    stream: MailStream,
    unread: Vec<u8>,
}

impl Wire {
    pub fn new(stream: MailStream) -> Self {
        Self { stream, unread: Vec::new() }
    }

    /// Reads one whole answer, following its continuation lines.
    pub async fn read_reply(&mut self, limit: Duration) -> Result<Reply, WireError> {
        let mut code = None;
        let mut lines = Vec::new();
        loop {
            let line = self.read_line(limit).await?;
            let Some(parsed) = parse_line(&line) else {
                return Err(WireError::Unreadable("a line that is not an answer".to_owned()));
            };
            if *code.get_or_insert(parsed.code) != parsed.code {
                return Err(WireError::Unreadable("an answer with two different codes".to_owned()));
            }
            lines.push(parsed.text.to_owned());
            if !parsed.more {
                return Ok(Reply { code: parsed.code, lines });
            }
            if lines.len() >= MAX_REPLY_LINES {
                return Err(WireError::Unreadable("an answer that never ends".to_owned()));
            }
        }
    }

    /// Writes one command line. A line break inside it is refused: nothing a
    /// person typed may start a command of its own.
    pub async fn send_line(&mut self, line: &str, limit: Duration) -> Result<(), WireError> {
        if line.contains(['\r', '\n']) {
            return Err(WireError::Unreadable("a command with a line break in it".to_owned()));
        }
        let mut bytes = Vec::with_capacity(line.len() + 2);
        bytes.extend_from_slice(line.as_bytes());
        bytes.extend_from_slice(b"\r\n");
        self.send_raw(&bytes, limit).await
    }

    /// Writes bytes as they are, and waits until they are on their way.
    pub async fn send_raw(&mut self, bytes: &[u8], limit: Duration) -> Result<(), WireError> {
        let write = async {
            self.stream.write_all(bytes).await?;
            self.stream.flush().await
        };
        match timeout(limit, write).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => Err(WireError::Io(error.to_string())),
            Err(_) => Err(WireError::TimedOut),
        }
    }

    /// The bare socket, for starting TLS. Refused when the server has sent
    /// anything beyond its last answer: those bytes arrived before the
    /// secure connection existed and must not be believed after it.
    pub fn into_socket(self) -> Result<TcpStream, WireError> {
        if !self.unread.is_empty() {
            return Err(WireError::Unreadable(
                "data sent along with the answer to STARTTLS".to_owned(),
            ));
        }
        self.stream
            .into_plain()
            .ok_or_else(|| WireError::Unreadable("the connection is already secure".to_owned()))
    }

    async fn read_line(&mut self, limit: Duration) -> Result<String, WireError> {
        let read = async {
            loop {
                if let Some(end) = self.unread.iter().position(|b| *b == b'\n') {
                    let raw: Vec<u8> = self.unread.drain(..=end).collect();
                    let text = String::from_utf8_lossy(&raw);
                    return Ok(text.trim_end_matches(['\r', '\n']).to_owned());
                }
                if self.unread.len() > MAX_LINE {
                    return Err(WireError::Unreadable("an answer line that is too long".to_owned()));
                }
                let mut chunk = [0u8; 2048];
                match self.stream.read(&mut chunk).await {
                    Ok(0) => return Err(WireError::Closed),
                    Ok(n) => self.unread.extend_from_slice(&chunk[..n]),
                    Err(error) => return Err(WireError::Io(error.to_string())),
                }
            }
        };
        timeout(limit, read).await.unwrap_or(Err(WireError::TimedOut))
    }
}

#[cfg(test)]
include!("wire.tests.rs");
