//! Who a message can be from: the accounts that may send mail, and which one
//! the message is from now.

use model::AccountId;

/// One account a message can be written from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sender {
    pub account: AccountId,
    /// The From line as it shows: `Name <address>`, or the address alone when
    /// the account has no name.
    pub from: String,
    /// The bare address, which a reply leaves out of its recipients.
    pub address: String,
    /// The signature the account's file asks for; empty for none.
    pub signature: String,
}

impl Sender {
    pub fn new(account: AccountId, name: &str, address: &str, signature: &str) -> Self {
        let name = name.trim();
        let from = if name.is_empty() { address.to_string() } else { format!("{name} <{address}>") };
        Self { account, from, address: address.to_string(), signature: signature.to_string() }
    }
}

/// The sender after `from` in the list, going `forward` or back and wrapping
/// round; `from` itself when there is nowhere else to go.
pub fn neighbour(count: usize, from: usize, forward: bool) -> usize {
    if count == 0 {
        return 0;
    }
    if forward { (from + 1) % count } else { (from + count - 1) % count }
}

#[cfg(test)]
include!("sender.tests.rs");
