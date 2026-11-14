//! A small HTTP/1.1 client for the token endpoint: one POST of a form, one
//! reply, `Connection: close`. HTTPS goes through the same TLS settings as the
//! mail connections (the bundled roots); plain HTTP is allowed only to this
//! machine. A redirect is an error, the reply is read to at most 256 KB, and
//! every wait has a deadline that names what was being awaited.

use std::time::Duration;

use rustls::pki_types::ServerName;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Instant, timeout_at};
use tokio_rustls::TlsConnector;

use super::http_parse::{Body, Head, MAX_HEAD, parse_head, read_body};
use crate::account_config::{Trust, is_loopback_host};
use crate::imap_session::{MailStream, classify_tls, client_config};

/// The most body bytes read from a reply.
pub const MAX_BODY: usize = 256 * 1024;

/// The waits of one request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpLimits {
    /// Reaching the server, including the secure handshake.
    pub connect: Duration,
    /// The whole request, from the first connection attempt to the last byte.
    pub total: Duration,
}

impl HttpLimits {
    pub const fn production() -> Self {
        Self { connect: Duration::from_secs(15), total: Duration::from_secs(30) }
    }

    /// The same wait for connecting and for the whole request.
    pub const fn within(limit: Duration) -> Self {
        Self { connect: limit, total: limit }
    }
}

/// An `http` or `https` address, cut apart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url {
    pub https: bool,
    /// The host without brackets.
    pub host: String,
    pub port: u16,
    /// The path and query, starting with `/`.
    pub target: String,
}

impl Url {
    /// Reads an absolute `http://` or `https://` address. No user name, no
    /// fragment, no blanks or control characters.
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err("it has a blank or a control character".to_owned());
        }
        let (https, rest) = if let Some(rest) = text.strip_prefix("https://") {
            (true, rest)
        } else if let Some(rest) = text.strip_prefix("http://") {
            (false, rest)
        } else {
            return Err("it must start with https://".to_owned());
        };
        if rest.contains('#') {
            return Err("it must not have a fragment".to_owned());
        }
        let (authority, target) = match rest.find(['/', '?']) {
            Some(at) if rest.as_bytes()[at] == b'/' => (&rest[..at], rest[at..].to_owned()),
            Some(at) => (&rest[..at], format!("/{}", &rest[at..])),
            None => (rest, "/".to_owned()),
        };
        if authority.contains('@') {
            return Err("it must not hold a user name".to_owned());
        }
        let (host, port) = split_authority(authority, https)?;
        Ok(Self { https, host, port, target })
    }

    /// The value of the `Host` header.
    fn host_header(&self) -> String {
        let host = if self.host.contains(':') { format!("[{}]", self.host) } else { self.host.clone() };
        let default = if self.https { 443 } else { 80 };
        if self.port == default { host } else { format!("{host}:{}", self.port) }
    }
}

fn split_authority(authority: &str, https: bool) -> Result<(String, u16), String> {
    let default = if https { 443 } else { 80 };
    let (host, port_text) = if let Some(inner) = authority.strip_prefix('[') {
        let (host, after) = inner.split_once(']').ok_or("the bracket is not closed")?;
        (host, after.strip_prefix(':'))
    } else {
        match authority.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (authority, None),
        }
    };
    if host.is_empty() {
        return Err("it has no host".to_owned());
    }
    let port = match port_text {
        None => default,
        Some(text) => text.parse::<u16>().ok().filter(|p| *p != 0).ok_or("the port is not valid")?,
    };
    Ok((host.to_owned(), port))
}

/// A reply the client accepted: its status and body.
#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub body: Vec<u8>,
}

/// Why a request did not give a reply to read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    /// The connection, the handshake or a wait failed.
    Network(String),
    /// The server's bytes are not an HTTP reply this client reads.
    Malformed(String),
    /// The answer was a redirect, which is never followed.
    Redirected(u16),
    /// Plain HTTP to a host that is not this machine.
    NotAllowed(String),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(text) => f.write_str(text),
            Self::Malformed(text) => write!(f, "the sign-in service sent a reply this program cannot read ({text})"),
            Self::Redirected(code) => write!(f, "the sign-in service answered with a redirect (HTTP {code}), which is not followed"),
            Self::NotAllowed(host) => write!(f, "plain HTTP is only allowed to this machine, not to {host}"),
        }
    }
}

/// POSTs `form` (already encoded) to `url` and reads the reply.
pub async fn post_form(url: &Url, form: &str, limits: &HttpLimits) -> Result<Response, HttpError> {
    if !url.https && !is_loopback_host(&url.host) {
        return Err(HttpError::NotAllowed(url.host.clone()));
    }
    let deadline = Instant::now() + limits.total;
    let mut stream = connect(url, limits, deadline).await?;
    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\n\
         Content-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n{form}",
        url.target,
        url.host_header(),
        form.len()
    );
    let wrote = async {
        stream.write_all(request.as_bytes()).await?;
        stream.flush().await
    };
    match timeout_at(deadline, wrote).await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => return Err(HttpError::Network(format!("{}: sending the request failed: {error}", url.host))),
        Err(_) => return Err(stalled(url, limits, "sending the request")),
    }
    read_reply(&mut stream, url, limits, deadline).await
}

fn stalled(url: &Url, limits: &HttpLimits, awaiting: &str) -> HttpError {
    HttpError::Network(format!(
        "{}: no answer within {} seconds while {awaiting}",
        url.host,
        limits.total.as_secs().max(1)
    ))
}

async fn connect(url: &Url, limits: &HttpLimits, deadline: Instant) -> Result<MailStream, HttpError> {
    let until = deadline.min(Instant::now() + limits.connect);
    let target = (url.host.as_str(), url.port);
    let tcp = match timeout_at(until, TcpStream::connect(target)).await {
        Ok(Ok(tcp)) => tcp,
        Ok(Err(error)) => return Err(HttpError::Network(format!("{}:{}: {error}", url.host, url.port))),
        Err(_) => return Err(stalled(url, &HttpLimits::within(limits.connect), "connecting")),
    };
    let _ = tcp.set_nodelay(true);
    if !url.https {
        return match tcp.peer_addr() {
            Ok(peer) if peer.ip().to_canonical().is_loopback() => Ok(MailStream::plain(tcp)),
            _ => Err(HttpError::NotAllowed(format!("{} (it led somewhere else)", url.host))),
        };
    }
    let config = client_config(Trust::Verify).map_err(HttpError::Network)?;
    let name = ServerName::try_from(url.host.clone())
        .map_err(|_| HttpError::Network(format!("{} is not a valid host name for a secure connection", url.host)))?;
    match timeout_at(until, TlsConnector::from(config).connect(name, tcp)).await {
        Ok(Ok(tls)) => Ok(MailStream::tls(tls)),
        Ok(Err(error)) => Err(HttpError::Network(format!("{}: {}", url.host, tls_words(&error)))),
        Err(_) => Err(stalled(url, &HttpLimits::within(limits.connect), "the secure handshake")),
    }
}

/// Why a handshake failed, in one phrase.
fn tls_words(error: &std::io::Error) -> String {
    use crate::imap_session::ConnectError;
    match classify_tls(error) {
        ConnectError::CertificateNotTrusted(text) => format!("the certificate is not trusted: {text}"),
        other => other.reason(),
    }
}

async fn read_reply(
    stream: &mut MailStream,
    url: &Url,
    limits: &HttpLimits,
    deadline: Instant,
) -> Result<Response, HttpError> {
    let malformed = |text: String| HttpError::Malformed(text);
    let mut buffer: Vec<u8> = Vec::new();
    let mut closed = false;
    let head: Head = loop {
        if let Some(head) = parse_head(&buffer).map_err(malformed)? {
            break head;
        }
        if closed || buffer.len() > MAX_HEAD {
            return Err(malformed("the reply ended or grew too long before its headers were complete".to_owned()));
        }
        closed = fill(stream, &mut buffer, url, limits, deadline).await?;
    };
    if (300..400).contains(&head.status) {
        return Err(HttpError::Redirected(head.status));
    }
    loop {
        match read_body(head.framing, &buffer[head.length..], closed, MAX_BODY).map_err(malformed)? {
            Body::Complete(body) => return Ok(Response { status: head.status, body }),
            Body::More => closed = fill(stream, &mut buffer, url, limits, deadline).await?,
        }
    }
}

/// Reads more bytes into `buffer`; true when the server closed its end.
async fn fill(
    stream: &mut MailStream,
    buffer: &mut Vec<u8>,
    url: &Url,
    limits: &HttpLimits,
    deadline: Instant,
) -> Result<bool, HttpError> {
    let mut chunk = [0u8; 8192];
    match timeout_at(deadline, stream.read(&mut chunk)).await {
        Ok(Ok(0)) => Ok(true),
        Ok(Ok(n)) => {
            buffer.extend_from_slice(&chunk[..n]);
            Ok(false)
        }
        // Many servers close without the TLS goodbye; the framing decides
        // whether the body is whole.
        Ok(Err(error)) if error.kind() == std::io::ErrorKind::UnexpectedEof => Ok(true),
        Ok(Err(error)) => Err(HttpError::Network(format!("{}: reading the reply failed: {error}", url.host))),
        Err(_) => Err(stalled(url, limits, "reading the reply")),
    }
}

#[cfg(test)]
include!("http.tests.rs");
