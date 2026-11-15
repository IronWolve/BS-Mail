//! Reaching the mail server: TCP, then TLS from the first byte when the
//! account says so, or after STARTTLS. Certificates are judged exactly as on
//! the mailbox side: against the bundled roots, or, for a loopback bridge, not
//! at all. A connection that may be plain or unverified must really lead to
//! this machine, whatever its name resolved to.

use std::time::Duration;

use rustls::pki_types::ServerName;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use super::error::SmtpError;
use super::timeouts::SmtpTimeouts;
use super::wire::Wire;
use crate::account_config::{Security, Trust};
use crate::account_smtp::SmtpSettings;
use crate::imap_session::{ConnectError, MailStream, classify_tls, client_config};

/// What this end calls itself in EHLO. A fixed literal, never this machine's
/// own address or name: some servers copy the name into a `Received` header
/// that every recipient can read, and the address of a home network is not
/// theirs to know.
pub const HELLO_NAME: &str = "[127.0.0.1]";

/// A connection with TLS settled for now, no greeting read yet.
pub struct Opened {
    pub wire: Wire,
    /// True when the connection is under TLS.
    pub secure: bool,
}

/// Connects to `host` and, for implicit TLS, secures the connection.
pub async fn open(
    settings: &SmtpSettings,
    host: &str,
    limits: &SmtpTimeouts,
) -> Result<Opened, SmtpError> {
    let bare = host.trim_matches(['[', ']']);
    let tcp = tcp_connect(settings, bare, limits.connect).await?;
    if settings.security == Security::Implicit {
        let stream = upgrade(settings, bare, tcp, limits.connect).await?;
        return Ok(Opened { wire: Wire::new(stream), secure: true });
    }
    Ok(Opened { wire: Wire::new(MailStream::plain(tcp)), secure: false })
}

/// Turns the connection into a secure one after the server agreed to
/// STARTTLS.
pub async fn start_tls(
    settings: &SmtpSettings,
    host: &str,
    wire: Wire,
    limit: Duration,
) -> Result<Wire, SmtpError> {
    let tcp = wire
        .into_socket()
        .map_err(|error| SmtpError::TlsFailed(format!("{error:?}")))?;
    let stream = upgrade(settings, host.trim_matches(['[', ']']), tcp, limit).await?;
    Ok(Wire::new(stream))
}

async fn tcp_connect(
    settings: &SmtpSettings,
    host: &str,
    limit: Duration,
) -> Result<TcpStream, SmtpError> {
    let attempt = tokio::time::timeout(limit, TcpStream::connect((host, settings.port))).await;
    let tcp = match attempt {
        Ok(Ok(tcp)) => tcp,
        Ok(Err(error)) => {
            return Err(SmtpError::Unreachable(format!("{host}:{}: {error}", settings.port)));
        }
        Err(_) => {
            return Err(SmtpError::Unreachable(format!(
                "{host}:{}: no answer within {} seconds",
                settings.port,
                limit.as_secs()
            )));
        }
    };
    ensure_local_when_unprotected(settings, host, &tcp)?;
    // Small commands should not wait for a full packet; failing is harmless.
    let _ = tcp.set_nodelay(true);
    Ok(tcp)
}

/// Plain or unverified is only acceptable to this machine, so check where the
/// name really led.
fn ensure_local_when_unprotected(
    settings: &SmtpSettings,
    host: &str,
    tcp: &TcpStream,
) -> Result<(), SmtpError> {
    if settings.security != Security::None && settings.trust != Trust::Any {
        return Ok(());
    }
    match tcp.peer_addr() {
        Ok(peer) if peer.ip().to_canonical().is_loopback() => Ok(()),
        Ok(peer) => Err(SmtpError::NotLoopback(format!("{host} led to {}", peer.ip()))),
        Err(error) => Err(SmtpError::Unreachable(error.to_string())),
    }
}

async fn upgrade(
    settings: &SmtpSettings,
    host: &str,
    tcp: TcpStream,
    limit: Duration,
) -> Result<MailStream, SmtpError> {
    let tls_config = client_config(settings.trust).map_err(SmtpError::TlsFailed)?;
    let name = ServerName::try_from(host.to_owned())
        .map_err(|_| SmtpError::TlsFailed("the host name is not valid for TLS".to_owned()))?;
    let handshake = TlsConnector::from(tls_config).connect(name, tcp);
    match tokio::time::timeout(limit, handshake).await {
        Ok(Ok(tls)) => Ok(MailStream::tls(tls)),
        Ok(Err(error)) => Err(match classify_tls(&error) {
            ConnectError::CertificateNotTrusted(text) => SmtpError::CertificateNotTrusted(text),
            ConnectError::Unreachable(text) => SmtpError::Unreachable(text),
            other => SmtpError::TlsFailed(other.reason()),
        }),
        Err(_) => Err(SmtpError::TlsFailed("the secure handshake timed out".to_owned())),
    }
}

#[cfg(test)]
include!("connect.tests.rs");
