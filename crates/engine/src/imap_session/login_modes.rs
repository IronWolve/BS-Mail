//! How a password may be sent to this server, found out before any
//! credential goes over the wire. A server that says LOGINDISABLED must not be
//! sent the password with LOGIN; it may take it with SASL PLAIN instead. The
//! question is asked on every connection, after TLS is settled: what a server
//! said before STARTTLS does not hold afterwards.

use std::time::Duration;

use async_imap::Client;
use async_imap::imap_proto::{Capability, Response, Status};
use tokio::io::AsyncWriteExt;

use super::error::ConnectError;
use super::stream::MailStream;

/// The tag of the one question. The library's own tags start with `A`.
const TAG: &str = "pre1";

/// How the sign-in may be sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// The LOGIN command.
    Login,
    /// AUTHENTICATE PLAIN.
    Plain,
    /// The server allows neither; the password is not sent.
    Refuse,
}

/// What the server said about password sign-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LoginModes {
    /// The server forbids the LOGIN command (LOGINDISABLED).
    pub login_disabled: bool,
    /// The server offers SASL PLAIN (AUTH=PLAIN).
    pub plain: bool,
}

impl LoginModes {
    pub fn from_capabilities(found: &[Capability<'_>]) -> Self {
        let mut modes = Self::default();
        for capability in found {
            match capability {
                Capability::Auth(name) if name.eq_ignore_ascii_case("PLAIN") => modes.plain = true,
                Capability::Atom(name) if name.eq_ignore_ascii_case("LOGINDISABLED") => {
                    modes.login_disabled = true;
                }
                _ => {}
            }
        }
        modes
    }

    /// How to send this user name and password. LOGIN is what every server
    /// takes, so it stays the choice unless the server forbids it. PLAIN is
    /// also used for a name or password with characters outside ASCII, which
    /// LOGIN cannot carry, when the server offers it.
    pub fn method_for(&self, user: &str, password: &str) -> Method {
        let plain_possible = self.plain && !user.contains('\0') && !password.contains('\0');
        if self.login_disabled {
            return if plain_possible { Method::Plain } else { Method::Refuse };
        }
        if plain_possible && !(user.is_ascii() && password.is_ascii()) {
            return Method::Plain;
        }
        Method::Login
    }
}

/// Asks CAPABILITY on a connection that is not signed in yet. A server that
/// answers NO or BAD is taken as saying nothing, which leaves LOGIN.
pub async fn ask(client: &mut Client<MailStream>, limit: Duration) -> Result<LoginModes, ConnectError> {
    match tokio::time::timeout(limit, exchange(client)).await {
        Ok(result) => result,
        Err(_) => Err(ConnectError::Unreachable("the server did not answer the capability question".to_owned())),
    }
}

async fn exchange(client: &mut Client<MailStream>) -> Result<LoginModes, ConnectError> {
    let line = format!("{TAG} CAPABILITY\r\n");
    let stream = client.get_mut();
    stream.write_all(line.as_bytes()).await.map_err(unreachable)?;
    stream.flush().await.map_err(unreachable)?;
    let mut modes = LoginModes::default();
    loop {
        let Some(response) = client.read_response().await.map_err(unreachable)? else {
            return Err(ConnectError::Unreachable("the server closed the connection".to_owned()));
        };
        match response.parsed() {
            Response::Capabilities(found) => modes = LoginModes::from_capabilities(found),
            Response::Done { tag, status, .. } if tag.0 == TAG => {
                return Ok(if *status == Status::Ok { modes } else { LoginModes::default() });
            }
            Response::Data { status: Status::Bye, .. } => {
                return Err(ConnectError::Unreachable("the server closed the connection".to_owned()));
            }
            _ => {}
        }
    }
}

fn unreachable(error: std::io::Error) -> ConnectError {
    ConnectError::Unreachable(error.to_string())
}

#[cfg(test)]
include!("login_modes.tests.rs");
