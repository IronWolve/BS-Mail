//! Carrying out a Search: the query is read, compiled to IMAP search keys,
//! and run as one `UID SEARCH` in each folder that is searched. The newest
//! hits of every folder are fetched as list rows (no body, nothing marked as
//! read), merged newest first across folders and cut to the most the screen
//! takes.
//!
//! It only reads: each folder is opened with EXAMINE, searched, and its
//! envelope rows fetched. The message ids of the rows are issued exactly as
//! for a folder's page, so a later body fetch or flag change finds the
//! message again. The connection is left with some other mailbox open; the
//! next command that needs the open folder opens it again.
//!
//! Exactly one event answers a search: the results, or a failure notice named
//! `search` with a plain sentence. When some folders were not searched (the
//! time ran out before they were reached, a server refused the search in
//! them, or they could not be opened) the results say so in `incomplete`
//! instead of passing for the whole answer. So do results that hold fewer
//! messages than the search found, because the newest of each folder and the
//! newest overall are all that is listed: `total` is every match, and
//! `incomplete` says how many of them are listed.

use std::time::Instant;

use model::{AccountId, Envelope, Event, FolderId, RequestId, Uid};

use super::search_folders::{Target, choose};
use super::worker::Worker;
use crate::imap_session::{
    ImapSession, LITERAL_REFUSAL, SENDS_LITERALS, SessionError, UidSet, build_envelope, fetched_row,
};
use crate::search::{Keys, Part, compile, parse};

/// The most messages one answer holds.
pub(super) const MAX_RESULTS: usize = 100;

/// The most hits fetched from one folder: its newest by UID.
const PER_FOLDER: usize = 100;

/// A found message and when it arrived, for merging folders.
struct Hit {
    arrived: i64,
    envelope: Envelope,
}

/// What a search over several folders came to.
#[derive(Default)]
struct Tally {
    total: u32,
    hits: Vec<Hit>,
    /// Folders that could be opened.
    opened: usize,
    /// Folders in which a search ran and was answered.
    searched: usize,
    /// The first reason a server refused a search or a fetch.
    refusal: Option<String>,
    /// Folders never reached because the search ran out of time.
    not_reached: usize,
    /// Folders in which the server refused the search or the fetch.
    refused_in: usize,
    /// Folders that could not be opened.
    unopened: usize,
}

impl Tally {
    /// What the search missed, in a sentence, or `None` when it covered all
    /// `folders`.
    fn incomplete(&self, folders: usize) -> Option<String> {
        let missed = self.not_reached + self.refused_in + self.unopened;
        if missed == 0 {
            return None;
        }
        let mut reasons = Vec::new();
        if self.not_reached > 0 {
            reasons.push(format!("{} not reached before the search ran out of time", self.not_reached));
        }
        if self.refused_in > 0 {
            let why = self.refusal.as_deref().map(|text| format!(" ({text})")).unwrap_or_default();
            reasons.push(format!("{} refused the search{why}", self.refused_in));
        }
        if self.unopened > 0 {
            reasons.push(format!("{} could not be opened", self.unopened));
        }
        let covered = folders.saturating_sub(missed);
        Some(format!("searched {covered} of {folders} folders, so messages may be missing: {}", reasons.join("; ")))
    }

    /// A sentence that says the list holds only `shown` of the matches, or
    /// `None` when it holds all of them.
    fn cut(&self, shown: usize) -> Option<String> {
        let total = usize::try_from(self.total).unwrap_or(usize::MAX);
        (total > shown).then(|| format!("showing only the newest {shown} of {total} matches; older ones are not listed"))
    }
}

/// The two notes of a search as one, in the order they were found.
fn joined(folders: Option<String>, cut: Option<String>) -> Option<String> {
    match (folders, cut) {
        (Some(folders), Some(cut)) => Some(format!("{folders}; {cut}")),
        (one, other) => one.or(other),
    }
}

impl Worker {
    /// Serves a Search. Only a lost connection is an error; every other
    /// outcome is the event that answers it.
    pub(super) async fn search(
        &mut self,
        session: &mut ImapSession,
        request: RequestId,
        account: &AccountId,
        query: &str,
    ) -> Result<Vec<Event>, SessionError> {
        if *account != self.account {
            let reason = format!("unknown account: {}", account.0);
            return Ok(vec![self.sink.failure("search", reason)]);
        }
        let (targets, keys) = match self.plan_search(query) {
            Ok(plan) => plan,
            Err(sentence) => return Ok(vec![self.sink.failure("search", sentence)]),
        };
        let mut tally = Tally::default();
        let started = Instant::now();
        for (at, target) in targets.iter().enumerate() {
            if at > 0 && started.elapsed() >= self.timing.search_deadline {
                tally.not_reached = targets.len() - at;
                break;
            }
            self.search_folder(session, target, &keys, &mut tally).await?;
        }
        let shown = tally.hits.len().min(MAX_RESULTS);
        let incomplete = joined(tally.incomplete(targets.len()), tally.cut(shown));
        Ok(vec![self.search_answer(request, tally, incomplete)])
    }

    /// The folders to search and the keys to search them with, or the
    /// sentence that says why this query cannot be run.
    fn plan_search(&self, query: &str) -> Result<(Vec<Target>, Keys), String> {
        let parsed = parse(query).map_err(|error| error.to_string())?;
        let keys = compile(parsed.expr.as_ref()).map_err(|error| error.to_string())?;
        if !SENDS_LITERALS && keys.parts().iter().any(|part| matches!(part, Part::Literal(_))) {
            return Err(LITERAL_REFUSAL.to_owned());
        }
        let targets = choose(self.state.folders(), &parsed.folders);
        if targets.is_empty() {
            return Err(match parsed.folders.as_slice() {
                [] => "there is no folder to search".to_owned(),
                texts => format!("no folder matches {}", texts.join(" and ")),
            });
        }
        Ok((targets, keys))
    }

    /// Searches one folder and adds its newest hits to `tally`. A folder
    /// that cannot be opened is skipped without a word; one whose server
    /// refuses the search or the fetch is skipped and the reason kept.
    async fn search_folder(
        &mut self,
        session: &mut ImapSession,
        target: &Target,
        keys: &Keys,
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        let info = match session.examine(&target.path).await {
            Ok(info) => info,
            Err(error) if error.is_lost() => return Err(error),
            Err(_) => {
                tally.unopened += 1;
                return Ok(());
            }
        };
        self.note_validity(target.id, info.uid_validity);
        tally.opened += 1;
        if info.exists == 0 {
            return Ok(());
        }
        let found = match session.uid_search(keys).await {
            Ok(found) => found,
            Err(error) if error.is_lost() => return Err(error),
            Err(error) => {
                tally.refusal.get_or_insert_with(|| error.text().to_owned());
                tally.refused_in += 1;
                return Ok(());
            }
        };
        tally.searched += 1;
        tally.total = tally.total.saturating_add(u32::try_from(found.len()).unwrap_or(u32::MAX));
        self.collect_rows(session, target.id, found, tally).await
    }

    /// Fetches the rows of the newest of `found` and adds them to `tally`.
    async fn collect_rows(
        &mut self,
        session: &mut ImapSession,
        folder: FolderId,
        found: Vec<u32>,
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        let newest: Vec<u32> = found.into_iter().rev().take(PER_FOLDER).collect();
        let Some(set) = UidSet::new(newest) else {
            return Ok(());
        };
        let rows = match session.fetch_rows(&set).await {
            Ok(rows) => rows,
            Err(error) if error.is_lost() => return Err(error),
            Err(error) => {
                tally.refusal.get_or_insert_with(|| error.text().to_owned());
                tally.refused_in += 1;
                return Ok(());
            }
        };
        for row in rows.iter().filter_map(fetched_row) {
            let id = self.state.message_id(folder, Uid(row.uid));
            let envelope = build_envelope(id, folder, &row);
            let arrived = row.internal_date.map_or(envelope.date.unix, |date| date.unix);
            tally.hits.push(Hit { arrived, envelope });
        }
        Ok(())
    }

    /// The event that answers the search.
    fn search_answer(&mut self, request: RequestId, mut tally: Tally, incomplete: Option<String>) -> Event {
        if tally.searched == 0 {
            if let Some(text) = &tally.refusal {
                let reason = format!("the server refused the search: {text}");
                return self.sink.failure("search", reason);
            }
            if tally.opened == 0 {
                return self.sink.failure("search", "no folder could be opened for the search");
            }
        }
        self.search_shown = true;
        Event::SearchResults {
            request,
            total: tally.total,
            envelopes: newest_first(&mut tally.hits),
            incomplete,
        }
    }
}

/// The hits, newest arrival first, at most `MAX_RESULTS`. Hits that arrived
/// at the same moment keep the order they were found in.
fn newest_first(hits: &mut Vec<Hit>) -> Vec<Envelope> {
    hits.sort_by_key(|hit| std::cmp::Reverse(hit.arrived));
    hits.drain(..).take(MAX_RESULTS).map(|hit| hit.envelope).collect()
}

#[cfg(test)]
include!("search.tests.rs");
