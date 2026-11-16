//! The second page's part of the account file text, and how an account's saved
//! values fill the page.
//!
//! The page writes only what has a value: a box left empty writes no key, so
//! the program's own choice stands. The engine checks every value when the
//! program reads the text (a folder path with a control character, a
//! certificate check turned off for a server that is not on this computer),
//! and the reason comes back to the form with the row it names. The password
//! and the secrets are not part of this page.

use super::form::SetupForm;
use super::prefill::AccountPrefill;
use super::settings_page::SettingsField;

/// Said when the poll box holds something that is not a number of seconds.
const POLL_WORDS: &str = "check how often to look for mail (`poll_seconds`): a whole number of seconds, 0 to never; \
     less than 15 counts as 15";

impl SetupForm {
    /// Adds the poll interval, the signature and the folders to the account
    /// file's table, for the boxes that have something in them.
    pub(super) fn put_settings(&self, table: &mut toml::Table) -> Result<(), String> {
        let poll = self.settings.poll.text();
        let poll = poll.trim();
        if !poll.is_empty() {
            let seconds: u32 = poll.parse().map_err(|_| POLL_WORDS.to_string())?;
            table.insert("poll_seconds".to_string(), i64::from(seconds).into());
        }
        let signature = self.settings.signature.text();
        if !signature.trim().is_empty() {
            table.insert("signature".to_string(), signature.into());
        }
        let mut folders = toml::Table::new();
        for (field, buffer) in SettingsField::FOLDERS.iter().zip(&self.settings.folders) {
            let path = buffer.text();
            let path = path.trim();
            if let (Some(key), false) = (field.folder_key(), path.is_empty()) {
                folders.insert(key.to_string(), path.to_string().into());
            }
        }
        if !folders.is_empty() {
            table.insert("folders".to_string(), folders.into());
        }
        Ok(())
    }

    /// Fills the second page with an account's saved values.
    pub(super) fn load_settings(&mut self, prefill: &AccountPrefill) {
        let page = &mut self.settings;
        page.poll.set_text(&prefill.poll_seconds.map(|seconds| seconds.to_string()).unwrap_or_default());
        page.signature.set_text(&prefill.signature);
        for (buffer, path) in page.folders.iter_mut().zip(&prefill.folders) {
            buffer.set_text(path);
        }
        page.sent_copy = prefill.save_sent;
        // The certificate checks follow the servers, as the form has always
        // had them do, until the user chooses one on this page.
    }
}

#[cfg(test)]
include!("settings_toml.tests.rs");
