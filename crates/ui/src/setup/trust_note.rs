//! What the form says about connections that are not fully checked. A server
//! on this computer (a mail bridge, say) has a certificate nobody else can
//! vouch for, so the form writes `trust = "any"` for it, and a server there
//! may also be reached with no encryption. Neither has a row on the form, so
//! one line under it names the hosts, instead of the choice being made
//! silently.

use super::fields::Security;
use super::form::SetupForm;
use super::local_host::is_local_host;

impl SetupForm {
    /// The incoming server is local and its saved certificate check, if any,
    /// is not being kept: the file will say `trust = "any"`.
    pub(super) fn imap_trust_any(&self) -> bool {
        let host = self.imap_host.text();
        let host = host.trim();
        is_local_host(host) && self.verified_imap_host.as_deref() != Some(host)
    }

    /// The same for the outgoing server, when there is one.
    pub(super) fn smtp_trust_any(&self) -> bool {
        let host = self.smtp_host.text();
        let host = host.trim();
        !host.is_empty() && is_local_host(host) && self.verified_smtp_host.as_deref() != Some(host)
    }

    /// One line for the form when it will write a local server's certificate
    /// check off or reach it with no encryption, naming the hosts; `None`
    /// when neither is so.
    pub fn local_trust_note(&self) -> Option<String> {
        let (mut unchecked, mut plain): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
        let servers = [
            (self.imap_host.text(), self.imap_security, self.imap_trust_any()),
            (self.smtp_host.text(), self.smtp_security, self.smtp_trust_any()),
        ];
        for (host, security, trust_any) in servers {
            let host = host.trim().to_string();
            let list = match security {
                Security::None if is_local_host(&host) => &mut plain,
                _ if trust_any => &mut unchecked,
                _ => continue,
            };
            if !list.contains(&host) {
                list.push(host);
            }
        }
        let mut sentences = Vec::new();
        if !unchecked.is_empty() {
            sentences.push(format!("Certificate checking is off for {}.", unchecked.join(" and ")));
        }
        if !plain.is_empty() {
            sentences.push(format!("Not encrypted: {}.", plain.join(" and ")));
        }
        (!sentences.is_empty()).then(|| sentences.join(" "))
    }
}

#[cfg(test)]
include!("trust_note.tests.rs");
