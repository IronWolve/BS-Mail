//! The walk that builds the safe copy: the parsed tree is read from the top
//! and a new text is written from it, element by element, keeping only what
//! the lists in `allow` name. The new text is made by this code from the
//! tree's names and values, with every value escaped, so nothing the sender
//! wrote reaches the page except as text, as an allowed attribute's value or
//! as the checked picture and address that replace the sender's.

use html5ever::ns;

use super::allow::{Policy, REFERENCES, attribute_kept, is_void, policy};
use super::css;
use super::dom::{Dom, Kind, NodeId};
use super::limits::{MAX_BODY_BYTES, MAX_DEPTH, MAX_KEPT_CSS_BYTES, MAX_PICTURES_TOTAL};
use super::urls::{content_id, inline_picture, is_remote, picture_address, safe_link};
use super::write;

/// What the walk made.
#[derive(Debug, Default)]
pub(super) struct Cleaned {
    /// The page's content.
    pub body: String,
    /// The kept style rules, for the head.
    pub styles: String,
    /// How many references to the network were left out.
    pub removed: u32,
}

/// The pictures that came with the message, by content id: their bytes.
pub(super) type Pictures<'a> = &'a dyn Fn(&str) -> Option<Vec<u8>>;

enum Work {
    Visit(NodeId, usize),
    Close(String),
}

struct Walk<'a> {
    dom: &'a Dom,
    pictures: Pictures<'a>,
    out: Cleaned,
    picture_bytes: usize,
}

/// Builds the safe copy of the tree.
pub(super) fn clean(dom: &Dom, pictures: Pictures<'_>) -> Cleaned {
    let mut walk = Walk { dom, pictures, out: Cleaned::default(), picture_bytes: 0 };
    let mut stack = vec![Work::Visit(dom.root(), 0)];
    while let Some(work) = stack.pop() {
        match work {
            Work::Close(name) => {
                walk.out.body.push_str("</");
                walk.out.body.push_str(&name);
                walk.out.body.push('>');
            }
            // Past the ceiling nothing more is opened; what is open is closed.
            Work::Visit(..) if walk.out.body.len() > MAX_BODY_BYTES => {}
            Work::Visit(id, depth) => walk.visit(id, depth, &mut stack),
        }
    }
    walk.out
}

impl Walk<'_> {
    fn visit(&mut self, id: NodeId, depth: usize, stack: &mut Vec<Work>) {
        let node = self.dom.node(id);
        match &node.kind {
            Kind::Text(text) => write::text(&mut self.out.body, text),
            Kind::Other => {}
            Kind::Document => push_children(stack, node.children.as_slice(), depth),
            Kind::Element { name, attrs } => {
                if name.ns != ns!(html) {
                    // Foreign content (svg, math) is never written.
                    self.out.removed += self.remote_in(id);
                    return;
                }
                let local: &str = &name.local;
                match policy(local) {
                    Policy::Drop => self.out.removed += self.remote_in(id),
                    Policy::Style => self.style(id),
                    Policy::Unwrap => {
                        // A `form` that is unwrapped still named an address to post to.
                        self.out.removed += count_remote(attrs);
                        push_children(stack, node.children.as_slice(), depth);
                    }
                    Policy::Keep if depth >= MAX_DEPTH => {}
                    Policy::Keep => self.keep(id, local, attrs, depth, stack),
                }
            }
        }
    }

    /// Writes a kept element's start tag with the attributes that may stay.
    fn keep(&mut self, id: NodeId, local: &str, attrs: &[(String, String)], depth: usize, stack: &mut Vec<Work>) {
        let picture = if local == "img" {
            match self.picture(attrs) {
                Some(address) => Some(address),
                None => return,
            }
        } else {
            None
        };
        let mut start = format!("<{local}");
        let mut linked = false;
        for (name, value) in attrs {
            let name = name.as_str();
            if name == "style" {
                let kept = css::declarations(value);
                self.out.removed += kept.remote;
                if !kept.css.is_empty() {
                    start.push_str(" style=\"");
                    write::attribute(&mut start, &kept.css);
                    start.push('"');
                }
            } else if local == "a" && name == "href" {
                if let Some(link) = safe_link(value) {
                    start.push_str(" href=\"");
                    write::attribute(&mut start, &link);
                    start.push('"');
                    linked = true;
                }
            } else if local == "img" && name == "src" {
                start.push_str(" src=\"");
                write::attribute(&mut start, picture.as_deref().unwrap_or_default());
                start.push('"');
            } else if attribute_kept(local, name) && value.len() <= 2000 {
                start.push(' ');
                start.push_str(name);
                start.push_str("=\"");
                write::attribute(&mut start, value);
                start.push('"');
            } else if REFERENCES.contains(&name) && is_remote(value) {
                self.out.removed += 1;
            }
        }
        if linked {
            start.push_str(" rel=\"noopener noreferrer\"");
        }
        start.push('>');
        self.out.body.push_str(&start);
        if !is_void(local) {
            stack.push(Work::Close(local.to_owned()));
            push_children(stack, self.dom.node(id).children.as_slice(), depth);
        }
    }

    /// The address a kept `img` shows: a small picture of the message itself
    /// (from a `cid:` reference or a `data:` address), never one from the network.
    fn picture(&mut self, attrs: &[(String, String)]) -> Option<String> {
        let src = &attrs.iter().find(|(name, _)| name == "src")?.1;
        if is_remote(src) {
            self.out.removed += 1;
            return None;
        }
        let address = match content_id(src) {
            Some(id) => (self.pictures)(&id).and_then(|bytes| picture_address(&bytes)),
            None => inline_picture(src),
        }?;
        self.picture_bytes += address.len();
        (self.picture_bytes <= MAX_PICTURES_TOTAL).then_some(address)
    }

    /// A style sheet: what the filter keeps goes to the head.
    fn style(&mut self, id: NodeId) {
        let text: String = self
            .dom
            .node(id)
            .children
            .iter()
            .filter_map(|child| match &self.dom.node(*child).kind {
                Kind::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect();
        let kept = css::sheet(&text);
        self.out.removed += kept.remote;
        if self.out.styles.len() + kept.css.len() <= MAX_KEPT_CSS_BYTES {
            self.out.styles.push_str(&kept.css);
        }
    }

    /// How many references to the network the subtree of a dropped element holds.
    fn remote_in(&self, root: NodeId) -> u32 {
        let (mut count, mut seen, mut pending) = (0u32, 0usize, vec![root]);
        while let Some(id) = pending.pop() {
            seen += 1;
            if seen > 100_000 {
                break;
            }
            let node = self.dom.node(id);
            if let Kind::Element { attrs, .. } = &node.kind {
                count += count_remote(attrs);
            }
            pending.extend(node.children.iter().copied());
        }
        count
    }
}

/// How many of an element's attributes name something on the network.
fn count_remote(attrs: &[(String, String)]) -> u32 {
    let found = attrs.iter().filter(|(name, value)| REFERENCES.contains(&name.as_str()) && is_remote(value)).count();
    u32::try_from(found).unwrap_or(u32::MAX)
}

/// Puts the children on the stack so the first is visited first.
fn push_children(stack: &mut Vec<Work>, children: &[NodeId], depth: usize) {
    for child in children.iter().rev() {
        stack.push(Work::Visit(*child, depth + 1));
    }
}

#[cfg(test)]
include!("sanitize.tests.rs");
