//! Providers whose whole preset is two server names, the outgoing port and a
//! line for the user: incoming mail is encrypted from the start (993). Data
//! only; the server names come from each provider's own help pages or, where
//! those could not be read, from several guides that agree, and the notes say
//! "this provider".

use super::fields::Security;

/// An app password is the way in.
const APP_PASSWORD: &str = "This provider needs an app password, made in your account's security settings.";
const APP_PASSWORD_ACCOUNT: &str = "This provider needs an app password, made in your account settings.";
/// Mail programs may have to be switched on first.
const SWITCH_ON: &str =
    "This provider may need IMAP switched on in its mail settings, and an app password if it asks for two-step sign-in.";
const SEPARATE_PASSWORD: &str = "This provider needs a separate email password for mail programs, made in its customer center.";
const IF_REFUSED: &str = "If this provider refuses your password, make an app password in your account's security settings.";
const GUESS_CHECK: &str = "If this does not connect, use the server names in your provider's mail settings.";
const EXTERNAL_PASSWORD: &str = "This provider needs a password for external programs, made in its security settings.";
const AUTH_CODE: &str =
    "This provider needs IMAP and SMTP switched on in its settings, and an authorization code in place of your password.";

/// One provider: its name in the provider list, the domains of its addresses
/// and its two servers.
pub(super) struct Simple {
    pub name: &'static str,
    pub domains: &'static [&'static str],
    pub imap: &'static str,
    pub smtp: &'static str,
    pub smtp_port: u16,
    pub smtp_security: Security,
    pub note: &'static str,
}

/// Outgoing mail on 587, encrypted after hello.
const fn simple(
    name: &'static str,
    domains: &'static [&'static str],
    imap: &'static str,
    smtp: &'static str,
    note: &'static str,
) -> Simple {
    Simple { name, domains, imap, smtp, smtp_port: 587, smtp_security: Security::Starttls, note }
}

/// Outgoing mail on 465, encrypted from the start.
const fn implicit(
    name: &'static str,
    domains: &'static [&'static str],
    imap: &'static str,
    smtp: &'static str,
    note: &'static str,
) -> Simple {
    Simple { name, domains, imap, smtp, smtp_port: 465, smtp_security: Security::Implicit, note }
}

static TABLE: [Simple; 24] = [
    simple(
        "Yahoo Mail",
        &["yahoo.com", "ymail.com", "rocketmail.com", "yahoo.co.uk", "yahoo.ca", "yahoo.de", "yahoo.fr", "yahoo.es", "yahoo.it", "yahoo.com.au"],
        "imap.mail.yahoo.com",
        "smtp.mail.yahoo.com",
        APP_PASSWORD,
    ),
    simple("iCloud Mail", &["icloud.com", "me.com", "mac.com"], "imap.mail.me.com", "smtp.mail.me.com", "This provider needs an app-specific password, made in your account settings."),
    simple("Fastmail", &["fastmail.com", "fastmail.fm"], "imap.fastmail.com", "smtp.fastmail.com", APP_PASSWORD_ACCOUNT),
    simple("GMX", &["gmx.com", "gmx.net", "gmx.de"], "imap.gmx.net", "mail.gmx.net", ""),
    simple("web.de", &["web.de"], "imap.web.de", "smtp.web.de", SWITCH_ON),
    simple("AOL Mail", &["aol.com"], "imap.aol.com", "smtp.aol.com", APP_PASSWORD),
    simple("Zoho Mail", &["zoho.com", "zohomail.com"], "imap.zoho.com", "smtp.zoho.com", SWITCH_ON),
    simple("mail.com", &["mail.com"], "imap.mail.com", "smtp.mail.com", SWITCH_ON),
    simple("T-Online", &["t-online.de"], "secureimap.t-online.de", "securesmtp.t-online.de", SEPARATE_PASSWORD),
    simple("Mailbox.org", &["mailbox.org"], "imap.mailbox.org", "smtp.mailbox.org", APP_PASSWORD_ACCOUNT),
    simple("Posteo", &["posteo.de", "posteo.net", "posteo.eu", "posteo.at", "posteo.ch", "posteo.org"], "posteo.de", "posteo.de", ""),
    simple("Comcast (Xfinity)", &["comcast.net"], "imap.comcast.net", "smtp.comcast.net", IF_REFUSED),
    // Most mailboxes here sit under the owner's own domain, which this guess
    // cannot see; these are the addresses the provider itself hands out.
    simple("1&1 / IONOS", &["ionos.com", "ionos.de", "ionos.co.uk", "ionos.es", "ionos.fr"], "imap.ionos.com", "smtp.ionos.com", GUESS_CHECK),
    simple("1&1 / IONOS", &["1and1.com", "1und1.de", "online.de"], "imap.ionos.com", "smtp.ionos.com", GUESS_CHECK),
    implicit("AT&T", &["att.net"], "imap.mail.att.net", "smtp.mail.att.net", APP_PASSWORD),
    simple("Cox", &["cox.net"], "imap.cox.net", "smtp.cox.net", IF_REFUSED),
    simple("Optimum", &["optimum.net"], "mail.optimum.net", "mail.optimum.net", GUESS_CHECK),
    implicit("Virgin Media", &["virginmedia.com", "blueyonder.co.uk", "ntlworld.com"], "imap.virginmedia.com", "smtp.virginmedia.com", APP_PASSWORD),
    simple("Free", &["free.fr"], "imap.free.fr", "smtp.free.fr", ""),
    implicit("SFR", &["sfr.fr"], "imap.sfr.fr", "smtp.sfr.fr", GUESS_CHECK),
    simple("La Poste", &["laposte.net"], "imap.laposte.net", "smtp.laposte.net", GUESS_CHECK),
    implicit("Mail.ru", &["mail.ru", "inbox.ru", "list.ru", "bk.ru"], "imap.mail.ru", "smtp.mail.ru", EXTERNAL_PASSWORD),
    implicit("QQ Mail", &["qq.com", "foxmail.com"], "imap.qq.com", "smtp.qq.com", AUTH_CODE),
    implicit("163 Mail", &["163.com"], "imap.163.com", "smtp.163.com", AUTH_CODE),
];

/// The provider whose addresses end in `domain`.
pub(super) fn find(domain: &str) -> Option<&'static Simple> {
    TABLE.iter().find(|entry| entry.domains.contains(&domain))
}

/// The provider the list calls `name`. Entries that share a name have the same
/// servers, so the first stands for all.
pub(super) fn find_named(name: &str) -> Option<&'static Simple> {
    TABLE.iter().find(|entry| entry.name == name)
}

/// The names of the providers here, each once, in table order.
pub(super) fn names() -> Vec<&'static str> {
    let mut all: Vec<&'static str> = Vec::new();
    for entry in &TABLE {
        if !all.contains(&entry.name) {
            all.push(entry.name);
        }
    }
    all
}

#[cfg(test)]
include!("simple_providers.tests.rs");
