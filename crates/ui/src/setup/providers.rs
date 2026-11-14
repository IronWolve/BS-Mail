//! Server details for the mail providers most people use, guessed from the
//! domain of the address. They are a starting point the user can change: the
//! form says so, and a domain not listed gets the usual `imap.` and `smtp.`
//! names.

use super::fields::Security;

/// What a provider needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preset {
    pub imap_host: String,
    pub imap_port: u16,
    pub imap_security: Security,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_security: Security,
    /// A line for the user, shown while the form is open: an app password
    /// is needed, a bridge must be running.
    pub note: &'static str,
    /// The name the engine knows this provider's browser sign-in by, when it
    /// has one.
    pub oauth: Option<&'static str>,
    /// Signing in through the browser is the way in, because a password no
    /// longer works here.
    pub browser_first: bool,
    /// The provider files a copy of mail sent through it, so the program must
    /// not add a second one.
    pub files_sent_itself: bool,
}

/// The preset for the domain of `email`, or the generic guess.
pub fn preset_for(email: &str) -> Option<Preset> {
    let domain = email.rsplit_once('@')?.1.trim().to_lowercase();
    if domain.is_empty() || domain.contains(char::is_whitespace) {
        return None;
    }
    Some(known(&domain).unwrap_or_else(|| generic(&domain)))
}

fn hosts(imap: &str, smtp: &str, smtp_port: u16, smtp_security: Security, note: &'static str) -> Preset {
    Preset {
        imap_host: imap.to_string(),
        imap_port: 993,
        imap_security: Security::Implicit,
        smtp_host: smtp.to_string(),
        smtp_port,
        smtp_security,
        note,
        oauth: None,
        browser_first: false,
        files_sent_itself: false,
    }
}

fn known(domain: &str) -> Option<Preset> {
    let tls = Security::Starttls;
    Some(match domain {
        "gmail.com" | "googlemail.com" => with_browser_sign_in(),
        "outlook.com" | "hotmail.com" | "live.com" | "msn.com" => browser_only(),
        "yahoo.com" | "ymail.com" | "rocketmail.com" => hosts(
            "imap.mail.yahoo.com",
            "smtp.mail.yahoo.com",
            587,
            tls,
            "This provider needs an app password, made in your account's security settings.",
        ),
        "icloud.com" | "me.com" | "mac.com" => hosts(
            "imap.mail.me.com",
            "smtp.mail.me.com",
            587,
            tls,
            "This provider needs an app-specific password, made in your account settings.",
        ),
        "fastmail.com" | "fastmail.fm" => hosts(
            "imap.fastmail.com",
            "smtp.fastmail.com",
            587,
            tls,
            "This provider needs an app password, made in your account settings.",
        ),
        "gmx.com" | "gmx.net" | "gmx.de" => hosts("imap.gmx.net", "mail.gmx.net", 587, tls, ""),
        "protonmail.com" | "proton.me" | "pm.me" | "protonmail.ch" => local_bridge(),
        _ => return None,
    })
}

/// The page where a program is registered with a provider, which is where a
/// client id comes from. Data only: the form opens it for the user.
pub fn registration_page(provider: &str) -> Option<&'static str> {
    match provider {
        "microsoft" => Some("https://entra.microsoft.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade"),
        "google" => Some("https://console.cloud.google.com/apis/credentials"),
        _ => None,
    }
}

/// A provider that takes an app password and also offers a sign-in through
/// the browser, which needs a client id the owner registers.
fn with_browser_sign_in() -> Preset {
    Preset {
        oauth: Some("google"),
        files_sent_itself: true,
        ..hosts(
            "imap.gmail.com",
            "smtp.gmail.com",
            587,
            Security::Starttls,
            "This provider needs an app password, not your usual one, or a sign-in through your browser (it needs a client id you register once).",
        )
    }
}

/// A provider that no longer takes a password for mail programs.
fn browser_only() -> Preset {
    Preset {
        oauth: Some("microsoft"),
        browser_first: true,
        files_sent_itself: true,
        ..hosts(
            "outlook.office365.com",
            "smtp.office365.com",
            587,
            Security::Starttls,
            "This provider no longer takes a password here: sign in through your browser. It needs a client id you register once (see docs/oauth-setup.md).",
        )
    }
}

/// A provider reached through a companion program on this computer.
fn local_bridge() -> Preset {
    Preset {
        imap_host: "127.0.0.1".to_string(),
        imap_port: 1143,
        imap_security: Security::Starttls,
        smtp_host: "127.0.0.1".to_string(),
        smtp_port: 1025,
        smtp_security: Security::Starttls,
        note: "This provider needs its companion mail program running on this computer; use the password that program shows you.",
        oauth: None,
        browser_first: false,
        files_sent_itself: false,
    }
}

/// The usual names for a domain nobody has listed.
fn generic(domain: &str) -> Preset {
    hosts(
        &format!("imap.{domain}"),
        &format!("smtp.{domain}"),
        587,
        Security::Starttls,
        "These server names are a guess: check them with your provider.",
    )
}

#[cfg(test)]
include!("providers.tests.rs");
