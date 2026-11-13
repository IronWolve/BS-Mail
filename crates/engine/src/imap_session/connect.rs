//! Opens the connection: TCP, then TLS from the start or after STARTTLS, then
//! the server's greeting. Signing in is `login.rs`.

use std::time::Duration;

use async_imap::Client;
use async_imap::imap_proto::{Response, Status};
use rustls::pki_types::ServerName;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use super::error::{ConnectError, classify_tls};
use super::stream::MailStream;
use super::timeouts::Timeouts;
use super::tls_config::client_config;
use crate::account_config::{AccountConfig, Security, Trust};

/// A connection with TLS settled and the greeting read, not yet signed in.
pub async fn open(
    config: &AccountConfig,
    timeouts: &Timeouts,
) -> Result<Client<MailStream>, ConnectError> {
    let tcp = tcp_connect(config, timeouts.connect).await?;
    match config.security {
        Security::Implicit => {
            let tls = upgrade(config, tcp, timeouts.connect).await?;
            let mut client = Client::new(tls);
            read_greeting(&mut client, timeouts.connect).await?;
            Ok(client)
        }
        Security::None => {
            let mut client = Client::new(MailStream::plain(tcp));
            read_greeting(&mut client, timeouts.connect).await?;
            Ok(client)
        }
        Security::Starttls => start_tls(config, tcp, timeouts.connect).await,
    }
}

async fn tcp_connect(config: &AccountConfig, limit: Duration) -> Result<TcpStream, ConnectError> {
    let target = (config.imap_host.trim_matches(['[', ']']), config.imap_port);
    let attempt = tokio::time::timeout(limit, TcpStream::connect(target)).await;
    let tcp = match attempt {
        Ok(Ok(tcp)) => tcp,
        Ok(Err(error)) => {
            return Err(ConnectError::Unreachable(format!(
                "{}:{}: {error}",
                config.imap_host, config.imap_port
            )));
        }
        Err(_) => {
            return Err(ConnectError::Unreachable(format!(
                "{}:{}: no answer within {} seconds",
                config.imap_host,
                config.imap_port,
                limit.as_secs()
            )));
        }
    };
    ensure_local_when_unprotected(config, &tcp)?;
    // Small commands should not wait for a full packet; failing is harmless.
    let _ = tcp.set_nodelay(true);
    Ok(tcp)
}

/// A plain or unverified connection is only acceptable to this machine, so
/// check where the name really led.
fn ensure_local_when_unprotected(
    config: &AccountConfig,
    tcp: &TcpStream,
) -> Result<(), ConnectError> {
    let unprotected = config.security == Security::None || config.trust == Trust::Any;
    if !unprotected {
        return Ok(());
    }
    match tcp.peer_addr() {
        Ok(peer) if peer.ip().to_canonical().is_loopback() => Ok(()),
        Ok(peer) => Err(ConnectError::NotLoopback(format!(
            "{} led to {}",
            config.imap_host,
            peer.ip()
        ))),
        Err(error) => Err(ConnectError::Unreachable(error.to_string())),
    }
}

async fn upgrade(
    config: &AccountConfig,
    tcp: TcpStream,
    limit: Duration,
) -> Result<MailStream, ConnectError> {
    let tls_config = client_config(config.trust).map_err(ConnectError::TlsFailed)?;
    let host = config.imap_host.trim_matches(['[', ']']).to_owned();
    let name = ServerName::try_from(host)
        .map_err(|_| ConnectError::TlsFailed("the host name is not valid for TLS".to_owned()))?;
    let handshake = TlsConnector::from(tls_config).connect(name, tcp);
    match tokio::time::timeout(limit, handshake).await {
        Ok(Ok(tls)) => Ok(MailStream::tls(tls)),
        Ok(Err(error)) => Err(classify_tls(&error)),
        Err(_) => Err(ConnectError::Unreachable(
            "the secure handshake timed out".to_owned(),
        )),
    }
}

async fn start_tls(
    config: &AccountConfig,
    tcp: TcpStream,
    limit: Duration,
) -> Result<Client<MailStream>, ConnectError> {
    let mut client = Client::new(MailStream::plain(tcp));
    read_greeting(&mut client, limit).await?;
    let asked = tokio::time::timeout(limit, client.run_command_and_check_ok("STARTTLS", None)).await;
    match asked {
        Ok(Ok(())) => {}
        Ok(Err(async_imap::error::Error::No(_) | async_imap::error::Error::Bad(_))) => {
            return Err(ConnectError::StartTlsMissing);
        }
        Ok(Err(error)) => return Err(ConnectError::Unreachable(error.to_string())),
        Err(_) => return Err(ConnectError::Unreachable("STARTTLS timed out".to_owned())),
    }
    let Some(tcp) = client.into_inner().into_plain() else {
        return Err(ConnectError::TlsFailed("the stream was already secure".to_owned()));
    };
    let tls = upgrade(config, tcp, limit).await?;
    Ok(Client::new(tls))
}

async fn read_greeting(
    client: &mut Client<MailStream>,
    limit: Duration,
) -> Result<(), ConnectError> {
    let read = tokio::time::timeout(limit, client.read_response()).await;
    let data = match read {
        Ok(Ok(Some(data))) => data,
        Ok(Ok(None)) => {
            return Err(ConnectError::Unreachable("closed before greeting".to_owned()));
        }
        Ok(Err(error)) => return Err(ConnectError::Unreachable(error.to_string())),
        Err(_) => return Err(ConnectError::Unreachable("no greeting".to_owned())),
    };
    match data.parsed() {
        Response::Data {
            status: Status::Ok, ..
        } => Ok(()),
        Response::Data {
            status: Status::Bye,
            outcome,
        } => Err(ConnectError::Greeting(
            outcome.information.as_deref().unwrap_or("goodbye").to_owned(),
        )),
        Response::Data {
            status: Status::PreAuth,
            ..
        } => Err(ConnectError::Greeting(
            "pre-authenticated sessions are not supported".to_owned(),
        )),
        _ => Err(ConnectError::Greeting("not an IMAP greeting".to_owned())),
    }
}
