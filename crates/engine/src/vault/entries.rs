//! The vault's contents in memory: names and secret texts. On disk (after
//! decrypting) it is a TOML table of name to string, each name a quoted key
//! such as `"accounts.main.password"`. The type never shows its contents in
//! debug output, and an error from reading it never quotes the text.

use std::collections::BTreeMap;
use std::fmt;

use model::Secret;

use super::error::VaultError;

#[derive(Default, Clone, PartialEq, Eq)]
pub(super) struct Entries(BTreeMap<String, Secret>);

impl fmt::Debug for Entries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entries(<{} redacted>)", self.0.len())
    }
}

impl Entries {
    pub fn get(&self, name: &str) -> Option<Secret> {
        self.0.get(name).cloned()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    pub fn set(&mut self, name: &str, value: Secret) {
        self.0.insert(name.to_owned(), value);
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.0.remove(name).is_some()
    }

    /// Removes every entry whose name starts with `prefix`; how many went.
    pub fn remove_prefix(&mut self, prefix: &str) -> usize {
        let before = self.0.len();
        self.0.retain(|name, _| !name.starts_with(prefix));
        before - self.0.len()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The text that gets encrypted. The caller overwrites it after use.
    pub fn to_plaintext(&self) -> Result<Vec<u8>, VaultError> {
        let table: BTreeMap<&str, &str> = self.0.iter().map(|(name, value)| (name.as_str(), value.expose())).collect();
        toml::to_string(&table).map(String::into_bytes).map_err(|_| VaultError::Contents)
    }

    /// The entries of decrypted text. Anything that is not a table of texts is
    /// refused without saying what it held.
    pub fn from_plaintext(bytes: &[u8]) -> Result<Self, VaultError> {
        let text = std::str::from_utf8(bytes).map_err(|_| VaultError::Contents)?;
        let table: BTreeMap<String, String> = toml::from_str(text).map_err(|_| VaultError::Contents)?;
        Ok(Self(table.into_iter().map(|(name, value)| (name, Secret::new(value))).collect()))
    }
}

#[cfg(test)]
include!("entries.tests.rs");
