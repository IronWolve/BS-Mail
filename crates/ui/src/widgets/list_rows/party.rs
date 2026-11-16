//! Whom a message row names. In the Sent and Drafts folders that is the
//! recipient, anywhere else the sender; a search result list mixes folders,
//! so each row goes by the role of the folder its own message is filed in.

use model::config::FromStyle;
use model::{Envelope, Folder, FolderRole};

use crate::format::format_address;

/// What a row of a message with no recipient names in the To column.
pub const NO_RECIPIENT: &str = "(no recipient)";

/// The labels the sender column can carry.
pub const FROM_LABEL: &str = "From";
pub const TO_LABEL: &str = "To";
pub const MIXED_LABEL: &str = "From/To";

/// Whether mail filed in a folder of `role` is shown by its recipient.
pub fn names_recipient(role: FolderRole) -> bool {
    matches!(role, FolderRole::Sent | FolderRole::Drafts)
}

/// The role of the folder `envelope` is filed in, `Plain` when the list of
/// folders has no such folder.
pub fn role_of(folders: &[Folder], envelope: &Envelope) -> FolderRole {
    folders
        .iter()
        .find(|folder| folder.id == envelope.folder)
        .map_or(FolderRole::Plain, |folder| folder.role)
}

/// The recipient as the column shows it: the first one by the sender
/// column's own style, and `+n` after it when there are `n` more.
pub fn recipient_text(envelope: &Envelope, style: FromStyle) -> String {
    let recipients = if envelope.to.is_empty() { &envelope.cc } else { &envelope.to };
    let Some(first) = recipients.first() else {
        return NO_RECIPIENT.to_owned();
    };
    let name = format_address(first, style);
    match recipients.len() {
        1 => name,
        more => format!("{name} +{}", more - 1),
    }
}

/// The header of the sender column over `envelopes`: `To` when every row
/// names its recipient, `From` when none does, and `From/To` over a mix. An
/// empty list goes by the role of the folder it is the list of.
pub fn column_label_for(folders: &[Folder], envelopes: &[Envelope], folder: Option<FolderRole>) -> &'static str {
    let recipients = envelopes.iter().filter(|envelope| names_recipient(role_of(folders, envelope))).count();
    match (recipients, envelopes.len()) {
        (0, 0) if folder.is_some_and(names_recipient) => TO_LABEL,
        (0, _) => FROM_LABEL,
        (some, all) if some == all => TO_LABEL,
        _ => MIXED_LABEL,
    }
}

#[cfg(test)]
include!("party.tests.rs");
