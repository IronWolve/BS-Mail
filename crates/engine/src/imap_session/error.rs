//! What can go wrong with a server connection, in words a person can act on.

use std::io;

use async_imap::error::Error as ImapError;

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
    /// The server refused the user name or password.
    AuthFailed(String),
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
        !matches!(self, Self::Unreachable(_) | Self::Greeting(_))
    }

    /// The operation name the footer shows with the reason.
    pub fn operation(&self) -> &'static str {
        match self {
            Self::AuthFailed(_) => "login",
            _ => "connect",
        }
    }

    /// One sentence for the footer.
    pub fn reason(&self) -> String {
        match self {
            Self::Unreachable(text) => format!("host unreachable: {text}"),
            Self::CertificateNotTrusted(text) => {
                format!("certificate not trusted: {text}")
            }
            Self::TlsFailed(text) => format!("secure connection failed: {text}"),
            Self::StartTlsMissing => "the server does not offer STARTTLS, so the password \
                 was not sent; use implicit TLS or fix the server"
                .to_owned(),
            Self::AuthFailed(text) => {
                format!("wrong user name or password ({text})")
            }
            Self::Greeting(text) => format!("the server did not accept the connection: {text}"),
            Self::NotLoopback(text) => {
                format!("refusing a plain or unverified connection to a non-local address: {text}")
            }
        }
    }
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
