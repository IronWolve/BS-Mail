//! The HTML parser's tree, as plain data: every node is a slot in one vector
//! and refers to the others by number, so a tree made by hostile input (tens
//! of thousands of tags deep, say) is dropped without recursion and walked
//! without it too. The parser is the one the text converter uses; this only
//! receives what it builds.

use std::borrow::Cow;
use std::cell::{Ref, RefCell};

use html5ever::interface::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{Attribute, ParseOpts, QualName, ns, parse_document};

/// A node's number in its tree.
pub(super) type NodeId = usize;

/// What a node is. Comments, processing instructions and the doctype are not
/// kept: nothing of them is ever written out.
#[derive(Debug)]
pub(super) enum Kind {
    Document,
    Text(String),
    Element { name: QualName, attrs: Vec<(String, String)> },
    /// A comment, a processing instruction or a doctype.
    Other,
}

#[derive(Debug)]
pub(super) struct Node {
    pub kind: Kind,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

/// The parsed document.
#[derive(Debug)]
pub(super) struct Dom {
    nodes: Vec<Node>,
}

impl Dom {
    pub fn root(&self) -> NodeId {
        0
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id]
    }
}

/// Parses `html` the way a browser would. The parser never fails: whatever
/// the input is, a tree comes out.
pub(super) fn parse(html: &str) -> Dom {
    let sink = Sink { nodes: RefCell::new(vec![Node { kind: Kind::Document, parent: None, children: Vec::new() }]) };
    // With scripting off, what is inside a `noscript` is parsed as markup, so
    // the pictures it holds (the usual tracking pixel) are found and counted.
    let options = ParseOpts {
        tree_builder: TreeBuilderOpts { scripting_enabled: false, ..Default::default() },
        ..Default::default()
    };
    parse_document(sink, options).one(StrTendril::from_slice(html))
}

struct Sink {
    nodes: RefCell<Vec<Node>>,
}

impl Sink {
    fn add(&self, kind: Kind) -> NodeId {
        let mut nodes = self.nodes.borrow_mut();
        nodes.push(Node { kind, parent: None, children: Vec::new() });
        nodes.len() - 1
    }

    /// Takes the node out of its parent's list.
    fn detach(&self, id: NodeId) {
        let mut nodes = self.nodes.borrow_mut();
        if let Some(parent) = nodes[id].parent.take() {
            nodes[parent].children.retain(|child| *child != id);
        }
    }

    /// Puts what the parser hands over at `at` in `parent`'s children (the end
    /// when `at` is `None`). Text next to text is joined.
    fn put(&self, parent: NodeId, at: Option<usize>, child: NodeOrText<NodeId>) {
        let id = match child {
            NodeOrText::AppendNode(id) => {
                self.detach(id);
                id
            }
            NodeOrText::AppendText(text) => {
                if self.join_text(parent, at, &text) {
                    return;
                }
                self.add(Kind::Text(text.to_string()))
            }
        };
        let mut nodes = self.nodes.borrow_mut();
        nodes[id].parent = Some(parent);
        let list = &mut nodes[parent].children;
        match at {
            Some(index) if index <= list.len() => list.insert(index, id),
            _ => list.push(id),
        }
    }

    /// Adds `text` to the text node just before position `at` when there is one.
    fn join_text(&self, parent: NodeId, at: Option<usize>, text: &str) -> bool {
        let mut nodes = self.nodes.borrow_mut();
        let list = &nodes[parent].children;
        let before = match at {
            Some(0) => return false,
            Some(index) => list.get(index - 1),
            None => list.last(),
        };
        let Some(previous) = before.copied() else {
            return false;
        };
        match &mut nodes[previous].kind {
            Kind::Text(existing) => {
                existing.push_str(text);
                true
            }
            _ => false,
        }
    }

    fn position(&self, sibling: NodeId) -> Option<(NodeId, usize)> {
        let nodes = self.nodes.borrow();
        let parent = nodes[sibling].parent?;
        let index = nodes[parent].children.iter().position(|child| *child == sibling)?;
        Some((parent, index))
    }
}

impl TreeSink for Sink {
    type Handle = NodeId;
    type Output = Dom;
    type ElemName<'a> = Ref<'a, QualName>;

    fn finish(self) -> Dom {
        Dom { nodes: self.nodes.into_inner() }
    }

    fn parse_error(&self, _message: Cow<'static, str>) {}

    fn get_document(&self) -> NodeId {
        0
    }

    fn elem_name<'a>(&'a self, target: &'a NodeId) -> Ref<'a, QualName> {
        Ref::map(self.nodes.borrow(), |nodes| match &nodes[*target].kind {
            Kind::Element { name, .. } => name,
            // The parser asks only about elements; an answer is still owed.
            _ => &EMPTY_NAME,
        })
    }

    fn create_element(&self, name: QualName, attrs: Vec<Attribute>, _flags: ElementFlags) -> NodeId {
        let attrs = attrs.into_iter().map(|attr| (attr.name.local.to_string(), attr.value.to_string())).collect();
        self.add(Kind::Element { name, attrs })
    }

    fn create_comment(&self, _text: StrTendril) -> NodeId {
        self.add(Kind::Other)
    }

    fn create_pi(&self, _target: StrTendril, _data: StrTendril) -> NodeId {
        self.add(Kind::Other)
    }

    fn append(&self, parent: &NodeId, child: NodeOrText<NodeId>) {
        self.put(*parent, None, child);
    }

    fn append_based_on_parent_node(&self, element: &NodeId, previous: &NodeId, child: NodeOrText<NodeId>) {
        if self.nodes.borrow()[*element].parent.is_some() {
            self.append_before_sibling(element, child);
        } else {
            self.append(previous, child);
        }
    }

    fn append_doctype_to_document(&self, _name: StrTendril, _public: StrTendril, _system: StrTendril) {}

    /// A template's contents are kept apart from the tree and never written.
    fn get_template_contents(&self, target: &NodeId) -> NodeId {
        *target
    }

    fn same_node(&self, x: &NodeId, y: &NodeId) -> bool {
        x == y
    }

    fn set_quirks_mode(&self, _mode: QuirksMode) {}

    fn append_before_sibling(&self, sibling: &NodeId, child: NodeOrText<NodeId>) {
        // A node that moves within one list leaves its old place first, so
        // the position of the sibling is worked out after.
        if let NodeOrText::AppendNode(id) = &child {
            self.detach(*id);
        }
        if let Some((parent, index)) = self.position(*sibling) {
            self.put(parent, Some(index), child);
        }
    }

    fn add_attrs_if_missing(&self, target: &NodeId, attrs: Vec<Attribute>) {
        let mut nodes = self.nodes.borrow_mut();
        if let Kind::Element { attrs: have, .. } = &mut nodes[*target].kind {
            for attr in attrs {
                let name = attr.name.local.to_string();
                if !have.iter().any(|(known, _)| *known == name) {
                    have.push((name, attr.value.to_string()));
                }
            }
        }
    }

    fn remove_from_parent(&self, target: &NodeId) {
        self.detach(*target);
    }

    fn reparent_children(&self, node: &NodeId, new_parent: &NodeId) {
        let moved = std::mem::take(&mut self.nodes.borrow_mut()[*node].children);
        let mut nodes = self.nodes.borrow_mut();
        for child in moved {
            nodes[child].parent = Some(*new_parent);
            nodes[*new_parent].children.push(child);
        }
    }
}

/// What `elem_name` answers for a node that is not an element.
static EMPTY_NAME: std::sync::LazyLock<QualName> =
    std::sync::LazyLock::new(|| QualName::new(None, ns!(), html5ever::LocalName::from("")));

#[cfg(test)]
include!("dom.tests.rs");
