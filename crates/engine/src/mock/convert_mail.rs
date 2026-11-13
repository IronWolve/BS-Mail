//! Turns fixture rows into the model's accounts, folders, envelopes and
//! status, checking the links between them (parents, folders, selection).

use std::collections::HashSet;

use model::{
    Account, AccountId, Address, ConnectionState, Envelope, Flags, Folder, FolderId, FolderRole,
    MessageId, Quota, SyncState, SyncStatus, ThreadId, Uid,
};

use super::convert_address::parse_address;
use super::convert_date::mail_date;
use super::fixture_error::FieldProblem;
use super::fixture_schema::{
    AccountRow, ConnectionText, FolderRow, MessageRow, RoleText, StatusRow, SyncText,
};

/// Accounts in file order. Exactly one must be marked selected.
pub(crate) fn accounts_from_rows(rows: &[AccountRow]) -> Result<Vec<Account>, FieldProblem> {
    if rows.is_empty() {
        return Err(FieldProblem::new("accounts", "needs at least one account"));
    }
    let selected = rows.iter().filter(|row| row.selected).count();
    if selected != 1 {
        return Err(FieldProblem::new(
            "accounts.selected",
            format!("exactly one account must be selected, found {selected}"),
        ));
    }
    Ok(rows
        .iter()
        .map(|row| Account {
            id: AccountId(row.id.clone()),
            name: row.name.clone(),
            address: row.address.clone(),
            state: connection_state(row.connection),
            selected: row.selected,
        })
        .collect())
}

fn connection_state(text: ConnectionText) -> ConnectionState {
    match text {
        ConnectionText::Online => ConnectionState::Online,
        ConnectionText::Connecting => ConnectionState::Connecting,
        ConnectionText::Offline => ConnectionState::Offline,
    }
}

/// Folders in file order, all owned by `account`. Ids must be unique and each
/// parent must be listed.
pub(crate) fn folders_from_rows(
    rows: &[FolderRow],
    account: &AccountId,
) -> Result<Vec<Folder>, FieldProblem> {
    let ids: HashSet<i64> = rows.iter().map(|row| row.id).collect();
    if ids.len() != rows.len() {
        return Err(FieldProblem::new("folders.id", "folder ids must be unique"));
    }
    let mut folders = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        if row.parent == Some(row.id) {
            return Err(FieldProblem::new(
                format!("folders[{index}].parent"),
                "a folder cannot be its own parent",
            ));
        }
        if let Some(parent) = row.parent.filter(|parent| !ids.contains(parent)) {
            return Err(FieldProblem::new(
                format!("folders[{index}].parent"),
                format!("folder {parent} is not in the fixture"),
            ));
        }
        folders.push(Folder {
            id: FolderId(row.id),
            account: account.clone(),
            path: row.path.clone(),
            name: row.name.clone(),
            parent: row.parent.map(FolderId),
            role: folder_role(row.role),
            unread: row.unread,
            total: row.total,
            expanded: row.expanded,
        });
    }
    Ok(folders)
}

fn folder_role(text: RoleText) -> FolderRole {
    match text {
        RoleText::Inbox => FolderRole::Inbox,
        RoleText::Starred => FolderRole::Starred,
        RoleText::Sent => FolderRole::Sent,
        RoleText::Drafts => FolderRole::Drafts,
        RoleText::Archive => FolderRole::Archive,
        RoleText::Alerts => FolderRole::Alerts,
        RoleText::Spam => FolderRole::Spam,
        RoleText::Trash => FolderRole::Trash,
        RoleText::Plain => FolderRole::Plain,
    }
}

/// Envelopes in file order (newest first). Ids must be unique and each folder
/// must be one of `folders`.
pub(crate) fn envelopes_from_rows(
    rows: &[MessageRow],
    folders: &[Folder],
) -> Result<Vec<Envelope>, FieldProblem> {
    let mut seen = HashSet::new();
    let mut envelopes = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let at = |name: &str| format!("messages[{index}].{name}");
        if !seen.insert(row.id) {
            return Err(FieldProblem::new(
                at("id"),
                format!("id {} is used twice", row.id),
            ));
        }
        if !folders
            .iter()
            .any(|folder| folder.id == FolderId(row.folder))
        {
            return Err(FieldProblem::new(
                at("folder"),
                format!("folder {} is not in the fixture", row.folder),
            ));
        }
        envelopes.push(envelope_from_row(row, &at)?);
    }
    Ok(envelopes)
}

fn envelope_from_row(
    row: &MessageRow,
    at: &dyn Fn(&str) -> String,
) -> Result<Envelope, FieldProblem> {
    let from = address_list(std::slice::from_ref(&row.from), &at("from"))?;
    Ok(Envelope {
        id: MessageId(row.id),
        folder: FolderId(row.folder),
        uid: Uid(row.uid),
        flags: Flags {
            seen: !row.unread,
            flagged: row.starred,
            alert: row.alert,
            ..Flags::default()
        },
        from,
        to: address_list(&row.to, &at("to"))?,
        cc: address_list(&row.cc, &at("cc"))?,
        subject: row.subject.clone(),
        date: mail_date(&row.date, &at("date"))?,
        size: row.size,
        has_attachments: row.attachments,
        tags: row.tags.clone(),
        thread: ThreadId(row.id),
        message_id: None,
    })
}

fn address_list(texts: &[String], field: &str) -> Result<Vec<Address>, FieldProblem> {
    texts
        .iter()
        .enumerate()
        .map(|(index, text)| {
            parse_address(text)
                .map_err(|problem| FieldProblem::new(format!("{field}[{index}]"), problem))
        })
        .collect()
}

/// The status block: sync time, counts, state and quota.
pub(crate) fn status_from_row(row: &StatusRow) -> Result<SyncStatus, FieldProblem> {
    let last_sync = match &row.last_sync {
        Some(value) => Some(mail_date(value, "status.last_sync")?.unix),
        None => None,
    };
    let quota = match (row.quota_used, row.quota_limit) {
        (Some(used), Some(limit)) => Some(Quota { used, limit }),
        (None, None) => None,
        _ => {
            return Err(FieldProblem::new(
                "status.quota_used",
                "quota_used and quota_limit go together",
            ));
        }
    };
    Ok(SyncStatus {
        last_sync,
        new_messages: row.new_messages,
        state: match row.state {
            SyncText::Idle => SyncState::Idle,
            SyncText::Syncing => SyncState::Syncing,
            SyncText::Failed => SyncState::Failed,
        },
        queue_len: row.queue_len,
        quota,
    })
}
