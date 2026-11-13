//! Mailbox storage figures (QUOTA), when the server offers them.

use async_imap::types::QuotaResourceName;
use model::Quota;

use super::error::SessionError;
use super::session::ImapSession;
use super::timeouts::bounded;

impl ImapSession {
    /// Used and limit bytes for the account's storage, or `None` when the
    /// server has no quota support or no storage limit. A refusal is not an
    /// error; a lost connection is.
    pub async fn storage_quota(&mut self) -> Result<Option<Quota>, SessionError> {
        if !self.caps.quota {
            return Ok(None);
        }
        let limit = self.limits.command;
        let result = bounded(limit, self.inner.get_quota_root("INBOX")).await;
        match self.settle(result) {
            Ok((_, quotas)) => Ok(quotas.iter().find_map(storage_of)),
            Err(error) if error.is_lost() => Err(error),
            Err(_) => Ok(None),
        }
    }
}

/// Quota resources count storage in units of 1024 bytes.
fn storage_of(quota: &async_imap::types::Quota) -> Option<Quota> {
    quota
        .resources
        .iter()
        .find(|resource| resource.name == QuotaResourceName::Storage)
        .map(|resource| Quota {
            used: resource.usage.saturating_mul(1024),
            limit: resource.limit.saturating_mul(1024),
        })
}

#[cfg(test)]
include!("quota.tests.rs");
