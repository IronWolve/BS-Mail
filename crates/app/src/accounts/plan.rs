//! What saving an account will do, worked out before anything is tried.
//!
//! The id is settled here, because the account's folder (and so the place its
//! sign-in tokens go) is named by it: a new account whose id is taken by a
//! different address gets `-2`, `-3`…, and an address that is already set up
//! is refused, so nothing is tried or written for it.

use std::fs;
use std::path::{Path, PathBuf};

use engine::AccountConfig;
use engine::vault::VaultView;

use super::id::{is_valid_id, unique_id};
use super::merge::carry_over;
use super::place::Place;
use super::reason::reason_of;

/// Said when a new account's address is already an account.
pub const ADDRESS_TAKEN: &str = "that address is already set up; pick Edit on it instead";

/// Where an account will be saved and under which id.
pub struct Plan {
    pub id: String,
    /// The account file's text with the final id in it. It can hold a
    /// password, so `Plan` is never printed whole.
    pub text: String,
    pub config: AccountConfig,
    /// The account file.
    pub file: PathBuf,
    /// The folder the account lives in, which is also where its tokens go.
    pub folder: PathBuf,
    /// The folder the vault and its key file are in; the account's passwords
    /// go there when it is saved.
    pub secrets: PathBuf,
    /// An existing account is being changed.
    pub edit: bool,
    /// The folder is not there yet, so this attempt is the one that makes it.
    pub new_folder: bool,
    /// A new account under the root: its file must not be there already.
    pub exclusive: bool,
    /// For a change to an account: the old file's text when the plan was made
    /// (`None`: there was no file), so saving can refuse if it changed since.
    pub old_text: Option<String>,
}

impl std::fmt::Debug for Plan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plan").field("id", &self.id).field("file", &self.file).field("edit", &self.edit).finish_non_exhaustive()
    }
}

impl Plan {
    /// The token file this account's browser sign-in writes, when it signs in
    /// that way.
    pub fn token_file(&self) -> Option<PathBuf> {
        self.config.clone().with_state_dir(&self.folder).oauth.map(|oauth| oauth.token_file)
    }
}

/// Works out the save of `text` into `place`. `edit` is the id of the account
/// being changed, `None` for a new one. `in_use` are the ids the running
/// accounts hold, which a new id must not be.
pub fn plan(place: &Place, edit: Option<&str>, in_use: &[String], text: &str) -> Result<Plan, String> {
    let config = AccountConfig::from_toml_str(text).map_err(|error| reason_of(&error))?;
    match (place, edit) {
        (Place::Root(_), Some(id)) => edit_in_root(place, id, config, text),
        (Place::Root(_), None) => add_to_root(place, in_use, config, text),
        (_, edit) => in_file(place, edit, config, text),
    }
}

/// `--account <file>`: exactly that file, whatever the id says.
fn in_file(place: &Place, edit: Option<&str>, config: AccountConfig, text: &str) -> Result<Plan, String> {
    if let Some(id) = edit
        && id != config.id
    {
        return Err(id_changed(id));
    }
    let (Some(file), Some(folder)) = (place.file_of(&config.id), place.folder_of(&config.id)) else {
        return Err("there is no place to save this account".to_string());
    };
    let (config, text, old_text) = if edit.is_some() {
        with_old_keys(place, &file, config, text.to_string())?
    } else {
        (config, text.to_string(), None)
    };
    Ok(Plan {
        id: config.id.clone(),
        text,
        config,
        file,
        folder,
        secrets: place.secrets_folder(),
        edit: edit.is_some(),
        new_folder: false,
        exclusive: false,
        old_text,
    })
}

fn edit_in_root(place: &Place, id: &str, config: AccountConfig, text: &str) -> Result<Plan, String> {
    if id != config.id {
        return Err(id_changed(id));
    }
    let scan = place.scan();
    if scan.has_address(&config.address, Some(id)) {
        return Err(ADDRESS_TAKEN.to_string());
    }
    finish(place, id.to_string(), config, text, true)
}

fn add_to_root(place: &Place, in_use: &[String], config: AccountConfig, text: &str) -> Result<Plan, String> {
    let scan = place.scan();
    if scan.has_address(&config.address, None) {
        return Err(ADDRESS_TAKEN.to_string());
    }
    let taken = |candidate: &str| {
        in_use.iter().any(|id| id == candidate)
            || place.file_of(candidate).is_none_or(|file| file.exists())
    };
    let id = unique_id(&config.id, taken).ok_or("too many accounts are named like that; change the address a little")?;
    finish(place, id, config, text, false)
}

fn finish(place: &Place, id: String, config: AccountConfig, text: &str, edit: bool) -> Result<Plan, String> {
    let (Some(file), Some(folder)) = (place.file_of(&id), place.folder_of(&id)) else {
        return Err(format!("\"{id}\" cannot be the name of an account folder"));
    };
    let (config, text) = if config.id == id { (config, text.to_string()) } else { with_id(&id, text)? };
    let (config, text, old_text) = if edit { with_old_keys(place, &file, config, text)? } else { (config, text, None) };
    let new_folder = !folder.exists();
    Ok(Plan { id, text, config, file, folder, secrets: place.secrets_folder(), edit, new_folder, exclusive: !edit, old_text })
}

/// For a change to an account that is already saved: the form's text with the
/// keys of the old file that the form cannot express carried over (a separate
/// sending login, how often to look for mail), and the account read from that
/// text. The account that is tried and installed is then the account that is
/// saved, not a plainer one that only the next start would correct.
///
/// The old file's text is returned too, so that saving can tell whether the
/// file changed since. A carried `secrets = "vault"` lets the account tried
/// read what the form does not give (a separate sending password) from the
/// vault, as it will be read once saved.
fn with_old_keys(
    place: &Place,
    file: &Path,
    config: AccountConfig,
    text: String,
) -> Result<(AccountConfig, String, Option<String>), String> {
    let Ok(old) = fs::read_to_string(file) else {
        return Ok((config, text, None));
    };
    let merged = carry_over(&old, &text);
    if merged == text {
        return Ok((config, text, Some(old)));
    }
    let vault = VaultView::open(&place.secrets_folder());
    let config = AccountConfig::from_toml_str_with_vault(&merged, &vault).map_err(|error| reason_of(&error))?;
    Ok((config, merged, Some(old)))
}

/// The same text and account with a different id.
fn with_id(id: &str, text: &str) -> Result<(AccountConfig, String), String> {
    if !is_valid_id(id) {
        return Err(format!("\"{id}\" cannot be the name of an account folder"));
    }
    let adjust = |text: &str| -> Option<String> {
        let mut table: toml::Table = toml::from_str(text).ok()?;
        table.insert("id".to_string(), id.to_string().into());
        toml::to_string(&table).ok()
    };
    let fresh = adjust(text).ok_or("the account could not be given another id")?;
    let config = AccountConfig::from_toml_str(&fresh).map_err(|error| reason_of(&error))?;
    Ok((config, fresh))
}

fn id_changed(id: &str) -> String {
    format!("the account being edited is \"{id}\" but the form says another id; close the form and try again")
}

#[cfg(test)]
include!("plan.tests.rs");
