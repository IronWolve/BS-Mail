//! Tags and the alert mark, from the keyboard. `t` opens the tag picker for
//! the marked messages, or the selected one (see `ask_tags` for the picker),
//! and `+` sets the alert mark on them, or clears it when they all have it.
//! Both are refused first, in words, on a read-only account. Each only asks
//! the backend: the rows change when it says it is done.

use std::collections::BTreeSet;

use model::{Command, FlagChange, MessageId};

use crate::app_view::AppView;
use crate::ask::tags::Held;
use crate::ask::{Ask, TagPicker};

impl AppView {
    /// The tags to offer for the account on screen: the ones seen on its mail
    /// before, and those of the messages shown now.
    fn tags_to_offer(&self) -> BTreeSet<String> {
        let mut tags = self.selection.account.as_ref().map(|account| self.mail.known_tags_of(account)).unwrap_or_default();
        let shown = self.mail.page.as_ref().map(|page| page.envelopes.as_slice()).unwrap_or_default();
        for tag in shown.iter().flat_map(|envelope| envelope.tags.iter()) {
            if !tags.iter().any(|have| have.eq_ignore_ascii_case(tag)) {
                tags.insert(tag.clone());
            }
        }
        tags
    }

    /// `t`: opens the picker for the targets.
    pub(crate) fn start_tag(&mut self) {
        let targets: Vec<(MessageId, Vec<String>)> =
            self.target_envelopes().iter().map(|envelope| (envelope.id, envelope.tags.clone())).collect();
        if targets.is_empty() {
            return;
        }
        if !self.changes_allowed_or_note() {
            return;
        }
        let held = |tag: &str| {
            let has = targets.iter().filter(|(_, tags)| tags.iter().any(|t| t.eq_ignore_ascii_case(tag))).count();
            match has {
                0 => Held::NoneOfThem,
                n if n == targets.len() => Held::All,
                _ => Held::Some,
            }
        };
        let known: Vec<(String, Held)> = self.tags_to_offer().into_iter().map(|tag| {
                let how = held(&tag);
                (tag, how)
            }).collect();
        let ids = targets.iter().map(|(id, _)| *id).collect();
        self.ask = Some(Ask::Tag(TagPicker::new(ids, known)));
    }

    /// `+`: sets the alert mark on the targets, or clears it when they all
    /// have it.
    pub(crate) fn toggle_alert(&mut self) {
        let targets = self.target_envelopes();
        if targets.is_empty() {
            return;
        }
        let make_alert = targets.iter().any(|envelope| !envelope.flags.alert);
        let ids: Vec<MessageId> = targets.iter().map(|envelope| envelope.id).collect();
        if !self.changes_allowed_or_note() {
            return;
        }
        self.queue.push(Command::SetFlags { messages: ids, change: FlagChange { alert: Some(make_alert), ..FlagChange::default() } });
    }
}

#[cfg(test)]
include!("tag_keys.tests.rs");
