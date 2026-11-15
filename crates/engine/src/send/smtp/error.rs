//! What can go wrong with a send, each case in one plain sentence a person
//! can act on. The password and the message text are never part of one:
//! what the server said is cleaned of controls, cut short and scrubbed of the
//! password before it is shown.

use std::fmt;

use crate::scrub::scrub;

use super::reply::Reply;
use super::wire::WireError;
use crate::send::check::Refusal;
use crate::sign_in_advice::sign_in_again;

/// The only reason given when the message may or may not have gone out.
pub const MAY_HAVE_BEEN_SENT: &str = "the connection dropped while sending, so the message \
     may or may not have been sent: check your Sent folder before trying again";

/// Which part of the conversation a failure happened in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Greeting,
    Hello,
    StartTls,
    SignIn,
    Sender,
    Recipients,
    StartingMessage,
    Whole,
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Greeting => "waiting for its greeting",
            Self::Hello => "introducing ourselves",
            Self::StartTls => "starting a secure connection",
            Self::SignIn => "signing in",
            Self::Sender => "giving the sender",
            Self::Recipients => "giving the recipients",
            Self::StartingMessage => "starting the message",
            Self::Whole => "doing the whole send",
        })
    }
}

/// A server's refusal, shortened for a sentence: its code and its words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Said {
    code: u16,
    text: String,
}

impl Said {
    /// The words are cleaned of controls and of every secret first, and only
    /// then cut, so a secret that straddles the cut is not left in part.
    pub fn of(reply: &Reply, secrets: &[String]) -> Self {
        let clean: String = reply.text().chars().filter(|c| !c.is_control()).collect();
        let secrets: Vec<&str> = secrets.iter().map(String::as_str).collect();
        let text: String = scrub(&clean, &secrets).chars().take(160).collect();
        Self { code: reply.code, text: text.trim().to_owned() }
    }

    /// True for a 4xx answer: the server says to try again later.
    pub fn is_temporary(&self) -> bool {
        self.code / 100 == 4
    }
}

impl fmt::Display for Said {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.code, self.text)?;
        Ok(())
    }
}

/// Why a send did not complete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmtpError {
    /// The account has no `smtp_host`.
    NoHost,
    Unreachable(String),
    CertificateNotTrusted(String),
    TlsFailed(String),
    StartTlsMissing,
    NotLoopback(String),
    /// The server hung up, or stopped answering, before the message began.
    Closed(Step),
    TimedOut(Step),
    Unreadable(String, Step),
    /// The server would not take the connection (its greeting or its answer
    /// to EHLO was a refusal).
    Declined(Said),
    SignInRefused(Said),
    SignInUnsupported(Vec<String>),
    /// The server refused the access token (or its error text was answered
    /// and it then said no).
    OAuthRefused(Said),
    /// The saved sign-in is gone or has ended: sign in again.
    SignInEnded(String),
    /// No access token just now (the provider could not be reached).
    TokenUnavailable(String),
    SenderRefused(String, Said),
    RecipientRefused(String, Said),
    /// The server's size limit, when it said one.
    TooBig(Option<u64>),
    MessageRefused(Said),
    /// The message was refused before anything was sent.
    Refused(Refusal),
    /// The connection failed after the message began to go out.
    MayHaveBeenSent,
}

impl SmtpError {
    /// The sentence for the failure notice.
    pub fn reason(&self) -> String {
        match self {
            Self::NoHost => "this account has no smtp_host in its file".to_owned(),
            Self::Unreachable(text) => format!("could not reach the mail server: {text}; nothing was sent"),
            Self::CertificateNotTrusted(text) => format!(
                "the mail server's certificate is not trusted ({text}), so nothing was sent"
            ),
            Self::TlsFailed(text) => format!(
                "a secure connection to the mail server could not be set up ({text}), \
                 so nothing was sent"
            ),
            Self::StartTlsMissing => "the mail server does not offer STARTTLS, so the password \
                 was not sent and nothing was sent; use implicit TLS or fix the server"
                .to_owned(),
            Self::NotLoopback(text) => format!(
                "refusing a plain or unverified connection to a non-local address ({text}); \
                 nothing was sent"
            ),
            Self::Closed(step) => format!("the mail server closed the connection while {step}; nothing was sent"),
            Self::TimedOut(step) => format!("the mail server did not answer in time while {step}; nothing was sent"),
            Self::Unreadable(text, step) => format!(
                "the mail server sent something this program could not read while {step} \
                 ({text}); nothing was sent"
            ),
            Self::Declined(said) => format!(
                "the mail server did not accept the connection ({said}); nothing was sent"
            ),
            Self::SignInRefused(_)
            | Self::SignInUnsupported(_)
            | Self::OAuthRefused(_)
            | Self::SignInEnded(_)
            | Self::TokenUnavailable(_) => self.sign_in_reason(),
            Self::SenderRefused(address, said) => format!(
                "the mail server refused the sender address {address} ({said}); nothing was sent"
            ),
            Self::RecipientRefused(address, said) => format!(
                "the mail server refused the recipient {address} ({said}); nothing was sent"
            ),
            Self::TooBig(Some(limit)) if *limit > 0 => format!(
                "the message is too big for this mail server, which takes at most {} bytes; \
                 nothing was sent",
                limit
            ),
            Self::TooBig(_) => "the message is too big for this mail server; nothing was sent".to_owned(),
            Self::MessageRefused(said) => format!(
                "the mail server did not accept the message ({said}); nothing was sent"
            ),
            Self::Refused(refusal) => refusal.reason().to_owned(),
            Self::MayHaveBeenSent => MAY_HAVE_BEEN_SENT.to_owned(),
        }
    }
}

impl SmtpError {
    /// The sentence for a failure of the sign-in itself.
    fn sign_in_reason(&self) -> String {
        match self {
            Self::SignInRefused(said) if said.is_temporary() => format!(
                "the mail server could not check the user name and password just now ({said}); \
                 nothing was sent"
            ),
            Self::SignInRefused(said) => format!(
                "the mail server refused the user name or password ({said}); nothing was sent"
            ),
            Self::SignInUnsupported(offered) => format!(
                "the mail server offers no sign-in method this program supports (it offers: {}); \
                 nothing was sent",
                offered.join(", ")
            ),
            Self::OAuthRefused(said) if said.is_temporary() => format!(
                "the mail server could not check the sign-in just now ({said}); nothing was sent"
            ),
            Self::OAuthRefused(said) => format!(
                "the mail server refused the sign-in ({said}); {}; nothing was sent",
                sign_in_again()
            ),
            Self::SignInEnded(words) => format!(
                "the sign-in was refused or has ended; {} ({words}); nothing was sent",
                sign_in_again()
            ),
            Self::TokenUnavailable(words) => format!(
                "could not get a sign-in token just now ({words}); nothing was sent"
            ),
            _ => String::new(),
        }
    }
}

/// A wire failure as the send's failure, for a step that comes before the
/// message begins.
pub fn from_wire(error: WireError, step: Step) -> SmtpError {
    match error {
        WireError::Closed => SmtpError::Closed(step),
        WireError::TimedOut => SmtpError::TimedOut(step),
        WireError::Io(text) | WireError::Unreadable(text) => SmtpError::Unreadable(text, step),
    }
}

#[cfg(test)]
include!("error.tests.rs");
