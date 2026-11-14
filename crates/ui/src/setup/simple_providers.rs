//! Providers whose whole preset is two server names and a line for the user:
//! encrypted from the start for incoming mail (993), encrypted after hello for
//! outgoing (587). Data only; the server names come from each provider's own
//! help pages, and the notes say "this provider".

/// An app password is the way in.
const APP_PASSWORD: &str = "This provider needs an app password, made in your account's security settings.";
const APP_PASSWORD_ACCOUNT: &str = "This provider needs an app password, made in your account settings.";
/// Mail programs may have to be switched on first.
const SWITCH_ON: &str =
    "This provider may need IMAP switched on in its mail settings, and an app password if it asks for two-step sign-in.";
const SEPARATE_PASSWORD: &str = "This provider needs a separate email password for mail programs, made in its customer center.";
const IF_REFUSED: &str = "If this provider refuses your password, make an app password in your account's security settings.";
const GUESS_CHECK: &str = "If this does not connect, use the server names in your provider's mail settings.";

/// One provider: the domains of its addresses and its two servers.
pub(super) struct Simple {
    pub domains: &'static [&'static str],
    pub imap: &'static str,
    pub smtp: &'static str,
    pub note: &'static str,
}

const fn simple(domains: &'static [&'static str], imap: &'static str, smtp: &'static str, note: &'static str) -> Simple {
    Simple { domains, imap, smtp, note }
}

static TABLE: [Simple; 14] = [
    simple(
        &["yahoo.com", "ymail.com", "rocketmail.com", "yahoo.co.uk", "yahoo.ca", "yahoo.de", "yahoo.fr", "yahoo.es", "yahoo.it", "yahoo.com.au"],
        "imap.mail.yahoo.com",
        "smtp.mail.yahoo.com",
        APP_PASSWORD,
    ),
    simple(&["icloud.com", "me.com", "mac.com"], "imap.mail.me.com", "smtp.mail.me.com", "This provider needs an app-specific password, made in your account settings."),
    simple(&["fastmail.com", "fastmail.fm"], "imap.fastmail.com", "smtp.fastmail.com", APP_PASSWORD_ACCOUNT),
    simple(&["gmx.com", "gmx.net", "gmx.de"], "imap.gmx.net", "mail.gmx.net", ""),
    simple(&["web.de"], "imap.web.de", "smtp.web.de", SWITCH_ON),
    simple(&["aol.com"], "imap.aol.com", "smtp.aol.com", APP_PASSWORD),
    simple(&["zoho.com", "zohomail.com"], "imap.zoho.com", "smtp.zoho.com", SWITCH_ON),
    simple(&["mail.com"], "imap.mail.com", "smtp.mail.com", SWITCH_ON),
    simple(&["t-online.de"], "secureimap.t-online.de", "securesmtp.t-online.de", SEPARATE_PASSWORD),
    simple(&["mailbox.org"], "imap.mailbox.org", "smtp.mailbox.org", APP_PASSWORD_ACCOUNT),
    simple(&["posteo.de", "posteo.net", "posteo.eu", "posteo.at", "posteo.ch", "posteo.org"], "posteo.de", "posteo.de", ""),
    simple(&["comcast.net"], "imap.comcast.net", "smtp.comcast.net", IF_REFUSED),
    // Most mailboxes here sit under the owner's own domain, which this guess
    // cannot see; these are the addresses the provider itself hands out.
    simple(&["ionos.com", "ionos.de", "ionos.co.uk", "ionos.es", "ionos.fr"], "imap.ionos.com", "smtp.ionos.com", GUESS_CHECK),
    simple(&["1and1.com", "1und1.de", "online.de"], "imap.ionos.com", "smtp.ionos.com", GUESS_CHECK),
];

/// The provider whose addresses end in `domain`.
pub(super) fn find(domain: &str) -> Option<&'static Simple> {
    TABLE.iter().find(|entry| entry.domains.contains(&domain))
}

#[cfg(test)]
include!("simple_providers.tests.rs");
