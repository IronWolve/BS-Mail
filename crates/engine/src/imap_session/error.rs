//! What can go wrong with a server connection, in words a person can act on.

use std::io;

use async_imap::error::Error as ImapError;

use crate::sign_in_advice::sign_in_again;
use crate::text_clean::single_line;

/// A failure to get from nothing to a signed-in session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectError {
    /// The host could not be reached (DNS, refused, timeout, reset). Worth
    /// retrying later.
    Unreachable(String),
    /// The certificate was rejected. Retrying cannot help.
    CertificateNotTrusted(String),
    /// The TLS handshake failed for another reason.
    TlsFailed(String),
    /// STARTTLS was asked for and the server does not do it.
    StartTlsMissing,
    /// The server refused the user name or password. Holds the server's own
    /// words, or nothing.
    AuthFailed(String),
    /// The access token could not be had just now (no network to the sign-in
    /// service, or it is down). Worth retrying later.
    SignInTrouble(String),
    /// An OAuth2 sign-in was refused by the server, or the saved sign-in is
    /// gone or ended. Holds the words behind it. Retrying cannot help.
    OAuthRefused(String),
    /// The server greeted with something that is not a usable IMAP session
    /// (a goodbye, a pre-authenticated session, garbage).
    Greeting(String),
    /// A host that must be local resolved to a non-local address.
    NotLoopback(String),
}

impl ConnectError {
    /// Retrying the same settings cannot succeed, so the worker stops until
    /// the user asks again.
    pub fn is_permanent(&self) -> bool {
        !matches!(self, Self::Unreachable(_) | Self::Greeting(_) | Self::SignInTrouble(_))
    }

    /// The operation name the footer shows with the reason.
    pub fn operation(&self) -> &'static str {
        match self {
            Self::AuthFailed(_) | Self::OAuthRefused(_) | Self::SignInTrouble(_) => "login",
            _ => "connect",
        }
    }

    /// One sentence for the footer.
    pub fn reason(&self) -> String {
        match self {
            Self::Unreachable(text) => format!("host unreachable: {text}"),
            Self::SignInTrouble(words) => format!("could not renew the sign-in just now, will try again ({words})"),
            Self::CertificateNotTrusted(text) => {
                format!("certificate not trusted: {text}")
            }
            Self::TlsFailed(text) => format!("secure connection failed: {text}"),
            Self::StartTlsMissing => "the server does not offer STARTTLS, so the password \
                 was not sent; use implicit TLS or fix the server"
                .to_owned(),
            Self::AuthFailed(words) if words.is_empty() => "wrong user name or password".to_owned(),
            Self::AuthFailed(words) => format!("wrong user name or password (the server said: {words})"),
            Self::OAuthRefused(words) => format!(
                "the sign-in was refused or has ended; {} ({words})",
                sign_in_again()
            ),
            Self::Greeting(text) => format!("the server did not accept the connection: {text}"),
            Self::NotLoopback(text) => {
                format!("refusing a plain or unverified connection to a non-local address: {text}")
            }
        }
    }
}

/// The server's own words in an error from the IMAP library, which wraps
/// them as `code: <code>, info: Some("<words>")`: only the words, with
/// control characters removed. Text that is not in that form is taken as the
/// words themselves.
pub fn server_words(text: &str) -> String {
    const OPEN: &str = "info: Some(\"";
    let words = match text.split_once(OPEN) {
        Some((_, rest)) => unquote(rest.strip_suffix("\")").unwrap_or(rest)),
        None if text.contains("info: None") => String::new(),
        None => text.to_owned(),
    };
    let one_line = single_line(&words);
    one_line.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(SERVER_WORDS_MAX).collect()
}

/// The most characters of a server's own words that are kept.
const SERVER_WORDS_MAX: usize = 200;

/// Undoes the escapes of Rust's debug form of a string: `\"` and `\\` are
/// the characters themselves, line breaks and tabs become blanks, and a
/// `\u{..}` escape is the character, or nothing when it is a control code.
fn unquote(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n' | 'r' | 't') | None => out.push(' '),
            Some('u') => {
                let hex: String = chars.by_ref().skip_while(|c| *c == '{').take_while(|c| *c != '}').collect();
                if let Some(decoded) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                    out.push(decoded);
                }
            }
            Some(other) => out.push(other),
        }
    }
    out
}

/// A failure of one operation on a signed-in session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    /// The connection is gone or stuck; reconnect.
    Lost(String),
    /// The server answered no, or said something unreadable; the connection
    /// is still fine.
    Refused(String),
}

impl SessionError {
    pub fn is_lost(&self) -> bool {
        matches!(self, Self::Lost(_))
    }

    /// The reason text, without the kind.
    pub fn text(&self) -> &str {
        match self {
            Self::Lost(text) | Self::Refused(text) => text,
        }
    }

    pub fn timed_out() -> Self {
        Self::Lost("the server stopped answering".to_owned())
    }
}

impl From<ImapError> for SessionError {
    fn from(error: ImapError) -> Self {
        match error {
            ImapError::Io(inner) => Self::Lost(inner.to_string()),
            ImapError::ConnectionLost => Self::Lost("the server closed the connection".to_owned()),
            ImapError::No(text) => Self::Refused(format!("the server said no: {text}")),
            ImapError::Bad(text) => Self::Refused(format!("the server rejected the request: {text}")),
            other => Self::Refused(other.to_string()),
        }
    }
}

impl From<io::Error> for SessionError {
    fn from(error: io::Error) -> Self {
        Self::Lost(error.to_string())
    }
}

/// Sorts a TLS handshake error into "certificate" and "other".
pub fn classify_tls(error: &io::Error) -> ConnectError {
    let inner = error
        .get_ref()
        .and_then(|e| e.downcast_ref::<rustls::Error>());
    match inner {
        Some(rustls::Error::InvalidCertificate(problem)) => {
            ConnectError::CertificateNotTrusted(describe_certificate(problem))
        }
        Some(other) => ConnectError::TlsFailed(other.to_string()),
        None if is_network_kind(error.kind()) => ConnectError::Unreachable(error.to_string()),
        None => ConnectError::TlsFailed(error.to_string()),
    }
}

fn describe_certificate(problem: &rustls::CertificateError) -> String {
    use rustls::CertificateError as C;
    match problem {
        C::UnknownIssuer => "it was not issued by a known authority".to_owned(),
        C::Expired | C::ExpiredContext { .. } => "it has expired".to_owned(),
        C::NotValidYet | C::NotValidYetContext { .. } => "it is not valid yet".to_owned(),
        C::NotValidForName | C::NotValidForNameContext { .. } => {
            "it is for a different host name".to_owned()
        }
        other => format!("{other:?}"),
    }
}

fn is_network_kind(kind: io::ErrorKind) -> bool {
    use io::ErrorKind as K;
    matches!(
        kind,
        K::ConnectionReset | K::ConnectionAborted | K::TimedOut | K::BrokenPipe | K::UnexpectedEof
    )
}

#[cfg(test)]
include!("error.tests.rs");
