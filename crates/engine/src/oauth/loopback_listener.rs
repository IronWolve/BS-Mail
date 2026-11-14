//! The listener the browser comes back to (RFC 8252): each connection served
//! on its own task, at most 8 KB read from each request under its own
//! deadline, and only
//! `GET /` with the right `state` counts. Everything else (a favicon, a wrong
//! state, anything odd) is answered 404 and the wait goes on. The wait as a
//! whole ends at its deadline. The listener is closed when the function
//! returns, whatever the outcome.

use std::sync::Arc;
use std::time::Duration;

use model::Secret;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::{Instant, timeout_at};

use super::error::provider_words;

/// The most bytes read from one request.
pub const MAX_REQUEST: usize = 8 * 1024;
/// The most connections served at the same time.
const MAX_OPEN: usize = 16;

const SIGNED_IN_PAGE: &str = "<!doctype html><meta charset=\"utf-8\"><title>Signed in</title>\
     <p>Signed in. You can close this tab.</p>";
const REFUSED_PAGE: &str = "<!doctype html><meta charset=\"utf-8\"><title>Not signed in</title>\
     <p>The sign-in was not completed. You can close this tab.</p>";

/// The waits of the listener.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListenLimits {
    /// The whole wait for the browser to come back.
    pub wait: Duration,
    /// One request, from accepting the connection to answering it.
    pub request: Duration,
}

/// What the browser brought.
#[derive(Debug)]
pub enum Redirect {
    /// The authorization code.
    Code(Secret),
    /// The provider's refusal, as its own cleaned words.
    Denied(String),
}

/// Why the wait ended without a redirect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListenError {
    /// The deadline passed.
    NoAnswer,
    /// The listening socket failed.
    Failed(String),
}

/// Waits for the browser's return with `state`, or until the deadline.
///
/// Each connection is served on its own task: a browser may open a spare
/// connection that never sends anything, and that must not hold up the one
/// that carries the redirect. At most `MAX_OPEN` are served at once; further
/// ones are closed without an answer.
pub async fn wait_for_redirect(
    listener: TcpListener,
    state: &str,
    limits: &ListenLimits,
) -> Result<Redirect, ListenError> {
    let until = Instant::now() + limits.wait;
    let (found, mut results) = tokio::sync::mpsc::unbounded_channel::<Redirect>();
    let room = Arc::new(Semaphore::new(MAX_OPEN));
    loop {
        tokio::select! {
            Some(redirect) = results.recv() => return Ok(redirect),
            accepted = timeout_at(until, listener.accept()) => {
                let accepted = accepted.map_err(|_| ListenError::NoAnswer)?;
                let socket = match accepted {
                    Ok((socket, _)) => socket,
                    Err(error) if is_transient(&error) => continue,
                    Err(error) => return Err(ListenError::Failed(format!("the local listener failed: {error}"))),
                };
                let Ok(permit) = Arc::clone(&room).try_acquire_owned() else {
                    continue;
                };
                let deadline = until.min(Instant::now() + limits.request);
                tokio::spawn(serve_one(socket, state.to_owned(), deadline, found.clone(), permit));
            }
        }
    }
}

/// Serves one connection under its own deadline and reports the redirect, if
/// it was one.
async fn serve_one(
    mut socket: TcpStream,
    state: String,
    deadline: Instant,
    found: tokio::sync::mpsc::UnboundedSender<Redirect>,
    _permit: OwnedSemaphorePermit,
) {
    if let Ok(Some(redirect)) = timeout_at(deadline, serve(&mut socket, &state)).await {
        let _ = found.send(redirect);
    }
}

fn is_transient(error: &std::io::Error) -> bool {
    use std::io::ErrorKind as K;
    matches!(error.kind(), K::ConnectionAborted | K::ConnectionReset | K::Interrupted | K::WouldBlock)
}

/// Reads one request and answers it. `Some` when it was the redirect.
async fn serve(socket: &mut TcpStream, state: &str) -> Option<Redirect> {
    let head = read_head(socket).await;
    let found = head.as_deref().and_then(|text| judge(text, state));
    let (status, page) = match &found {
        Some(Redirect::Code(_)) => ("200 OK", Some(SIGNED_IN_PAGE)),
        Some(Redirect::Denied(_)) => ("200 OK", Some(REFUSED_PAGE)),
        None => ("404 Not Found", None),
    };
    let _ = respond(socket, status, page).await;
    found
}

/// The request line and headers, up to the blank line; `None` when they do
/// not fit in `MAX_REQUEST` bytes or the client went away first.
async fn read_head(socket: &mut TcpStream) -> Option<String> {
    let mut buffer: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 1024];
    loop {
        if let Some(end) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
            return String::from_utf8(buffer[..end].to_vec()).ok();
        }
        if buffer.len() >= MAX_REQUEST {
            return None;
        }
        let room = (MAX_REQUEST - buffer.len()).min(chunk.len());
        match socket.read(&mut chunk[..room]).await {
            Ok(0) | Err(_) => return None,
            Ok(n) => buffer.extend_from_slice(&chunk[..n]),
        }
    }
}

async fn respond(socket: &mut TcpStream, status: &str, page: Option<&str>) -> std::io::Result<()> {
    let body = page.unwrap_or("");
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\
         Cache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\n\r\n",
        body.len()
    );
    socket.write_all(head.as_bytes()).await?;
    socket.write_all(body.as_bytes()).await?;
    socket.flush().await?;
    socket.shutdown().await
}

/// Decides what a request head is: the redirect, or nothing of interest.
fn judge(head: &str, state: &str) -> Option<Redirect> {
    let line = head.lines().next()?;
    let mut parts = line.split(' ');
    let (method, target, version) = (parts.next()?, parts.next()?, parts.next()?);
    if method != "GET" || parts.next().is_some() || !matches!(version, "HTTP/1.1" | "HTTP/1.0") {
        return None;
    }
    let (path, query) = target.split_once('?')?;
    if path != "/" {
        return None;
    }
    let pairs = parse_query(query);
    let value = |name: &str| pairs.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str());
    if value("state") != Some(state) {
        return None;
    }
    if let Some(error) = value("error") {
        return Some(Redirect::Denied(provider_words(error, value("error_description"), &[])));
    }
    value("code").filter(|code| !code.is_empty()).map(|code| Redirect::Code(Secret::new(code)))
}

/// `a=b&c=d` as decoded pairs.
fn parse_query(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (name, value) = part.split_once('=').unwrap_or((part, ""));
            (percent_decode(name), percent_decode(value))
        })
        .collect()
}

/// Undoes percent-encoding (and `+` for a blank); a stray `%` stays as it is.
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at] {
            b'+' => out.push(b' '),
            b'%' if at + 2 < bytes.len() && hex(bytes[at + 1]).is_some() && hex(bytes[at + 2]).is_some() => {
                out.push(hex(bytes[at + 1]).unwrap_or(0) << 4 | hex(bytes[at + 2]).unwrap_or(0));
                at += 2;
            }
            other => out.push(other),
        }
        at += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
include!("loopback_listener.tests.rs");
