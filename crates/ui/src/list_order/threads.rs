//! Grouping loaded messages into conversations.
//!
//! A message belongs with the ones it names in `References` and `In-Reply-To`
//! (the ids of its ancestors, oldest first), with the ones that name it, and,
//! when a message has no such ids at all, with messages of the same subject
//! within 30 days when one of them is a reply. Ancestors that are not loaded
//! still tie their loaded answers together (two replies to one absent
//! message are one conversation).
//!
//! The headers come from outside and can be hostile, so the walk is bounded:
//! at most `MAX_REFERENCES` ids are read from one message, a conversation
//! never holds more than `MAX_THREAD` messages, an id two messages of
//! different subjects both claim ties nobody, and nothing recurses: the
//! groups are found with a disjoint-set forest, in time near linear in the
//! messages and ids read. A cycle (A names B, B names A) or a message that
//! names itself is no problem for it.

use std::collections::HashMap;

use model::thread_headers::MAX_REFERENCES;
use model::{Envelope, normal_id};

use super::subject_key::{base_subject, has_reply_prefix};

/// The most messages one conversation holds; a larger one is split.
pub const MAX_THREAD: usize = 1000;

/// How far apart in time two messages with no ids may be to be one
/// conversation by their subject.
pub const SUBJECT_WINDOW_SECS: i64 = 30 * 24 * 60 * 60;

/// Disjoint sets over messages (and the ids of messages that are not loaded),
/// where a set may not grow past `MAX_THREAD` messages.
struct Forest {
    parent: Vec<usize>,
    /// Loaded messages in the set of each root.
    weight: Vec<usize>,
}

impl Forest {
    fn new(messages: usize) -> Self {
        Self { parent: (0..messages).collect(), weight: vec![1; messages] }
    }

    /// A node for an id no loaded message has.
    fn add_ghost(&mut self) -> usize {
        self.parent.push(self.parent.len());
        self.weight.push(0);
        self.parent.len() - 1
    }

    fn find(&mut self, mut node: usize) -> usize {
        while self.parent[node] != node {
            self.parent[node] = self.parent[self.parent[node]];
            node = self.parent[node];
        }
        node
    }

    /// Joins the sets of `a` and `b`, unless that would make a conversation
    /// larger than `MAX_THREAD`.
    fn join(&mut self, a: usize, b: usize) {
        let (a, b) = (self.find(a), self.find(b));
        if a == b || self.weight[a] + self.weight[b] > MAX_THREAD {
            return;
        }
        let (big, small) = if self.weight[a] >= self.weight[b] { (a, b) } else { (b, a) };
        self.parent[small] = big;
        self.weight[big] += self.weight[small];
    }
}

/// What an id is, among the ids of the loaded messages.
enum IdNode {
    /// The node of the message (or messages with one subject) that has it.
    Node(usize),
    /// Claimed by messages of different subjects: it ties nobody.
    Claimed,
}

/// The conversations among `envelopes`, as lists of indexes into it. Each
/// list is in date order, the oldest first (equal dates in the order given);
/// the lists are in the order of their first message in `envelopes`. A
/// message that is alone is a list of one.
pub fn group(envelopes: &[Envelope]) -> Vec<Vec<usize>> {
    let subjects: Vec<String> = envelopes.iter().map(|envelope| base_subject(&envelope.subject)).collect();
    let mut forest = Forest::new(envelopes.len());
    let mut ids = own_ids(envelopes, &subjects, &mut forest);
    for (at, envelope) in envelopes.iter().enumerate() {
        tie_to_ancestors(at, envelope, &mut ids, &mut forest);
    }
    tie_by_subject(envelopes, &subjects, &mut forest);
    collect(envelopes, &mut forest)
}

/// The id each message has, and the node it stands for. A second message with
/// an id joins the first's set only when their subjects are the same.
fn own_ids(envelopes: &[Envelope], subjects: &[String], forest: &mut Forest) -> HashMap<String, IdNode> {
    let mut ids: HashMap<String, IdNode> = HashMap::with_capacity(envelopes.len());
    for (at, envelope) in envelopes.iter().enumerate() {
        let Some(id) = envelope.message_id.as_deref().and_then(normal_id) else {
            continue;
        };
        match ids.get(&id) {
            None => {
                ids.insert(id, IdNode::Node(at));
            }
            Some(IdNode::Node(first)) if subjects[*first] == subjects[at] => forest.join(*first, at),
            Some(_) => {
                ids.insert(id, IdNode::Claimed);
            }
        }
    }
    ids
}

/// The ids of the messages `envelope` answers: its `References` (the first
/// and the last ones, when it has more than are read), else its
/// `In-Reply-To`.
fn ancestor_ids(envelope: &Envelope) -> Vec<String> {
    let refs = &envelope.threading.references;
    let source: Vec<&String> = if refs.len() > MAX_REFERENCES {
        let tail = refs.len() - (MAX_REFERENCES - 1);
        std::iter::once(&refs[0]).chain(&refs[tail..]).collect()
    } else if refs.is_empty() {
        envelope.threading.in_reply_to.iter().take(MAX_REFERENCES).collect()
    } else {
        refs.iter().collect()
    };
    source.into_iter().filter_map(|text| normal_id(text)).collect()
}

/// Ties message `at` to the chain of its ancestors, and the chain together:
/// each id in `References` is the parent of the next, and the last is the
/// message's own parent. The `In-Reply-To` id is a parent too.
fn tie_to_ancestors(at: usize, envelope: &Envelope, ids: &mut HashMap<String, IdNode>, forest: &mut Forest) {
    let chain = ancestor_ids(envelope);
    let mut previous: Option<usize> = None;
    for id in &chain {
        let Some(node) = node_of(id, ids, forest) else {
            continue;
        };
        if let Some(before) = previous {
            forest.join(before, node);
        }
        previous = Some(node);
    }
    if let Some(parent) = previous {
        forest.join(parent, at);
    }
    for id in envelope.threading.in_reply_to.iter().take(1).filter_map(|text| normal_id(text)) {
        if let Some(node) = node_of(&id, ids, forest) {
            forest.join(node, at);
        }
    }
}

/// The node of an id: the message that has it, or a node made for the id.
fn node_of(id: &str, ids: &mut HashMap<String, IdNode>, forest: &mut Forest) -> Option<usize> {
    match ids.get(id) {
        Some(IdNode::Node(node)) => Some(*node),
        Some(IdNode::Claimed) => None,
        None => {
            let node = forest.add_ghost();
            ids.insert(id.to_owned(), IdNode::Node(node));
            Some(node)
        }
    }
}

/// Ties messages that name no other message by their subject: those with the
/// same subject (less its prefixes) no more than 30 days apart, when one of
/// the two is a reply.
fn tie_by_subject(envelopes: &[Envelope], subjects: &[String], forest: &mut Forest) {
    let mut bare: Vec<usize> = (0..envelopes.len())
        .filter(|&at| {
            let headers = &envelopes[at].threading;
            headers.is_empty() && !subjects[at].is_empty()
        })
        .collect();
    bare.sort_by(|&a, &b| {
        subjects[a].cmp(&subjects[b]).then(envelopes[a].date.unix.cmp(&envelopes[b].date.unix)).then(a.cmp(&b))
    });
    for pair in bare.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let near = envelopes[b].date.unix.saturating_sub(envelopes[a].date.unix) <= SUBJECT_WINDOW_SECS;
        let reply = has_reply_prefix(&envelopes[a].subject) || has_reply_prefix(&envelopes[b].subject);
        if subjects[a] == subjects[b] && near && reply {
            forest.join(a, b);
        }
    }
}

/// The sets as lists, each in date order, in the order of their first message.
fn collect(envelopes: &[Envelope], forest: &mut Forest) -> Vec<Vec<usize>> {
    let mut place: HashMap<usize, usize> = HashMap::new();
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for at in 0..envelopes.len() {
        let root = forest.find(at);
        let index = *place.entry(root).or_insert_with(|| {
            groups.push(Vec::new());
            groups.len() - 1
        });
        groups[index].push(at);
    }
    for members in &mut groups {
        members.sort_by_key(|&at| (envelopes[at].date.unix, at));
    }
    groups
}

#[cfg(test)]
include!("threads.tests.rs");
