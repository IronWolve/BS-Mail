//! The keys in force: each action's own keys, with what the `[keys]` table
//! rebinds laid over them. A rebinding that cannot be kept is reported and
//! left out, and the action keeps the key it had; the rest are kept.

use std::sync::OnceLock;

use model::config::{KeyValue, KeysConfig};
use surface::KeyEvent;

use super::actions::{NOT_REBINDABLE, REBINDABLE, Rebindable, index_of};
use super::rules::refusal;
use super::spec::KeySpec;
use crate::input::Action;
use crate::input::action::fixed_action;

#[derive(Debug, Clone)]
pub struct KeyMap {
    /// The keys of each action of `REBINDABLE`, in force.
    keys: Vec<Vec<KeySpec>>,
    /// Which actions the table rebound.
    rebound: Vec<bool>,
    problems: Vec<String>,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self::new(&KeysConfig::default())
    }
}

impl KeyMap {
    /// The map for what the table says. Every entry that cannot be kept is in
    /// `problems`, with the reason, and its action keeps its old key.
    pub fn new(config: &KeysConfig) -> Self {
        let mut problems = Vec::new();
        let mut asked: Vec<Option<Vec<KeySpec>>> = vec![None; REBINDABLE.len()];
        for (name, value) in config.iter() {
            match read_entry(name, value) {
                Ok((index, keys)) => asked[index] = Some(keys),
                Err(problem) => problems.push(problem),
            }
        }
        settle_conflicts(&mut asked, &mut problems);
        let keys = REBINDABLE
            .iter()
            .zip(&asked)
            .map(|(entry, asked)| asked.clone().unwrap_or_else(|| entry.keys.to_vec()))
            .collect();
        let rebound = asked.iter().map(Option::is_some).collect();
        Self { keys, rebound, problems }
    }

    /// The map nothing has rebound, shared.
    pub fn standard() -> &'static KeyMap {
        static STANDARD: OnceLock<KeyMap> = OnceLock::new();
        STANDARD.get_or_init(Self::default)
    }

    /// What was wrong with the table, one line each.
    pub fn problems(&self) -> &[String] {
        &self.problems
    }

    /// The action a key press does on the main screen.
    pub fn action(&self, key: &KeyEvent) -> Option<Action> {
        let found = self.keys.iter().position(|keys| keys.iter().any(|spec| spec.matches(key)));
        match found {
            Some(index) => Some(REBINDABLE[index].action),
            None => fixed_action(key),
        }
    }

    /// The keys an action has now (empty for a name that is none).
    pub fn keys_of(&self, name: &str) -> &[KeySpec] {
        index_of(name).and_then(|index| self.keys.get(index)).map_or(&[], Vec::as_slice)
    }

    /// The key or keys of an action as text for the help box: `r`, or `r/F3`
    /// when it has more than one.
    pub fn text_of(&self, name: &str) -> String {
        self.keys_of(name).iter().map(KeySpec::text).collect::<Vec<_>>().join("/")
    }

    /// The text of the action's keycap in the key bar, when the table moved
    /// the action; `None` when the bar's own text is right.
    pub fn cap_text(&self, name: &str) -> Option<String> {
        let index = index_of(name)?;
        if !self.rebound.get(index).copied().unwrap_or(false) {
            return None;
        }
        self.keys.get(index)?.first().map(KeySpec::cap_text)
    }

    /// Every action with the key or keys it has now, as the table would write
    /// them: what `--print-config` shows, and it reads back as the same map.
    pub fn in_force(&self) -> KeysConfig {
        let mut table = KeysConfig::default();
        for (entry, keys) in REBINDABLE.iter().zip(&self.keys) {
            let texts: Vec<String> = keys.iter().map(KeySpec::text).collect();
            let value = if texts.len() == 1 { KeyValue::One(texts[0].clone()) } else { KeyValue::Many(texts) };
            table.0.insert(entry.name.to_string(), value);
        }
        table
    }
}

/// What the old keys of an action are called in a report.
fn old_keys(entry: &Rebindable) -> String {
    entry.keys.iter().map(KeySpec::text).collect::<Vec<_>>().join(" ")
}

/// One entry of the table: the action it names and the keys it asks for.
fn read_entry(name: &str, value: &KeyValue) -> Result<(usize, Vec<KeySpec>), String> {
    let Some(index) = index_of(name) else {
        let why = NOT_REBINDABLE.iter().find(|(known, _)| *known == name).map(|(_, why)| format!("cannot be changed: {why}"));
        return Err(format!("keys.{name}: {}", why.unwrap_or_else(|| "no action has this name".to_string())));
    };
    let entry = &REBINDABLE[index];
    let stays = format!("; {} stays", old_keys(entry));
    let Some(texts) = value.texts() else {
        return Err(format!("keys.{name}: write a key, or a list of keys, as text{stays}"));
    };
    if texts.is_empty() {
        return Err(format!("keys.{name}: no key was given{stays}"));
    }
    let mut keys: Vec<KeySpec> = Vec::new();
    for text in texts {
        let spec = KeySpec::parse(text).map_err(|reason| format!("keys.{name}: {reason}{stays}"))?;
        if !entry.fixed.contains(&spec)
            && let Some(reason) = refusal(&spec)
        {
            return Err(format!("keys.{name}: {reason}{stays}"));
        }
        if !keys.contains(&spec) {
            keys.push(spec);
        }
    }
    Ok((index, keys))
}

/// Takes back every rebinding whose key another action also has, until no key
/// is shared. An action that is not rebound has its own keys, and the keys it
/// always has (F2 for composing): a rebinding that asks for one of those is
/// the one given up. Two rebindings that ask for the same key are both given
/// up, since neither has more right to it.
fn settle_conflicts(asked: &mut [Option<Vec<KeySpec>>], problems: &mut Vec<String>) {
    loop {
        let claims = claims_of(asked);
        let clashes: Vec<(usize, KeySpec, usize)> = (0..asked.len())
            .filter_map(|index| {
                let keys = asked[index].as_ref()?;
                keys.iter().find_map(|spec| {
                    claims.iter().find(|(other, claimed)| *other != index && claimed == spec).map(|(other, _)| (index, *spec, *other))
                })
            })
            .collect();
        if clashes.is_empty() {
            return;
        }
        for (index, spec, other) in clashes {
            let (me, them) = (&REBINDABLE[index], &REBINDABLE[other]);
            problems.push(format!("keys.{}: {} is already the key of {}; {} stays", me.name, spec.text(), them.name, old_keys(me)));
            asked[index] = None;
        }
    }
}

/// Every key claimed, with the action (by place) that claims it.
fn claims_of(asked: &[Option<Vec<KeySpec>>]) -> Vec<(usize, KeySpec)> {
    let mut claims = Vec::new();
    for (index, entry) in REBINDABLE.iter().enumerate() {
        let own = asked[index].as_deref().unwrap_or(entry.keys);
        claims.extend(own.iter().chain(entry.fixed).map(|spec| (index, *spec)));
    }
    claims
}

#[cfg(test)]
include!("map.tests.rs");
