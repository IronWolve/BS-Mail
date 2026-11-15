//! Server details for the mail providers most people use, guessed from the
//! domain of the address. They are a starting point the user can change: the
//! form says so, and a domain not listed gets the usual `imap.` and `smtp.`
//! names.

use super::ascii_domain::ascii_domain;
use super::fields::Security;
use super::simple_providers::{find, find_named, names};

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

/// The preset for the domain of `email`, or the generic guess. A domain with
/// letters outside ASCII is guessed under its ASCII name, the only kind a
/// server name may have; one that has no ASCII name gets no guess.
pub fn preset_for(email: &str) -> Option<Preset> {
    let domain = email.rsplit_once('@')?.1.trim().to_lowercase();
    if domain.is_empty() || domain.contains(char::is_whitespace) {
        return None;
    }
    let domain = ascii_domain(&domain)?;
    Some(known(&domain).unwrap_or_else(|| generic(&domain)))
}

/// The three providers with a preset of their own, by the names the provider
/// list shows.
const GOOGLE_NAME: &str = "Gmail (Google)";
const MICROSOFT_NAME: &str = "Outlook / Hotmail (Microsoft)";
const PROTON_NAME: &str = "Proton Mail Bridge";

/// The preset of a provider chosen from the list by name, whatever the
/// address looks like: a mailbox under the owner's own domain is the usual
/// reason to choose.
pub fn preset_named(name: &str) -> Option<Preset> {
    match name {
        GOOGLE_NAME => Some(with_browser_sign_in()),
        MICROSOFT_NAME => Some(browser_only()),
        PROTON_NAME => Some(local_bridge()),
        _ => find_named(name).map(|entry| hosts(entry.imap, entry.smtp, entry.smtp_port, entry.smtp_security, entry.note)),
    }
}

/// The names the provider list offers, each once, in alphabetical order.
pub fn provider_names() -> Vec<&'static str> {
    let mut all = vec![GOOGLE_NAME, MICROSOFT_NAME, PROTON_NAME];
    all.extend(names());
    all.sort_by_key(|name| name.to_lowercase());
    all
}

/// The usual `imap.` and `smtp.` names for the domain of `email`, for a
/// provider that is none of the listed ones; no guess without a usable domain.
pub fn generic_preset_for(email: &str) -> Option<Preset> {
    let domain = email.rsplit_once('@')?.1.trim().to_lowercase();
    if domain.is_empty() || domain.contains(char::is_whitespace) {
        return None;
    }
    Some(generic(&ascii_domain(&domain)?))
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

/// Microsoft's consumer mail domains, which have no password sign-in left.
const BROWSER_ONLY_DOMAINS: [&str; 17] = [
    "outlook.com", "hotmail.com", "live.com", "msn.com", "hotmail.co.uk", "hotmail.fr", "hotmail.de", "hotmail.it",
    "hotmail.es", "outlook.fr", "outlook.de", "outlook.es", "outlook.it", "live.co.uk", "live.fr", "live.de", "live.it",
];

fn known(domain: &str) -> Option<Preset> {
    Some(match domain {
        "gmail.com" | "googlemail.com" => with_browser_sign_in(),
        _ if BROWSER_ONLY_DOMAINS.contains(&domain) => browser_only(),
        "protonmail.com" | "proton.me" | "pm.me" | "protonmail.ch" => local_bridge(),
        _ => {
            let entry = find(domain)?;
            hosts(entry.imap, entry.smtp, entry.smtp_port, entry.smtp_security, entry.note)
        }
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
        note: "This needs the Proton Mail Bridge program running on this computer. Use the user name and password Bridge shows you, not your Proton account password.",
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
