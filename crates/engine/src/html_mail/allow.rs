//! What may stay in the safe copy of a message. Everything here is an
//! allow-list: an element or an attribute that is not named is not written,
//! so a tag or attribute nobody thought of is dropped, never passed through.

/// What to do with an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Policy {
    /// Written, with the attributes that are allowed on it, and its children.
    Keep,
    /// Not written, but its children are (the page's own `html`, `head` and
    /// `body`, a `form` whose fields are gone, and anything unknown).
    Unwrap,
    /// Not written, and nothing inside it either.
    Drop,
    /// A style sheet: kept only as far as the filter in `css` allows.
    Style,
}

/// Elements that are written, with their children.
const KEPT: &[&str] = &[
    "a", "abbr", "address", "article", "aside", "b", "bdi", "bdo", "big", "blockquote", "br", "caption", "center",
    "cite", "code", "col", "colgroup", "dd", "del", "dfn", "div", "dl", "dt", "em", "figcaption", "figure", "font",
    "footer", "h1", "h2", "h3", "h4", "h5", "h6", "header", "hr", "i", "img", "ins", "kbd", "li", "main", "mark",
    "nav", "ol", "p", "pre", "q", "s", "samp", "section", "small", "span", "strike", "strong", "sub", "sup",
    "table", "tbody", "td", "tfoot", "th", "thead", "time", "tr", "tt", "u", "ul", "var", "wbr",
];

/// Elements that are dropped with everything inside them: scripts and frames,
/// everything that embeds or loads something, form controls, the page's
/// settings (`meta`, `link`, `base`, `title`), and the two kinds of foreign
/// content a script can hide in.
const DROPPED: &[&str] = &[
    "script", "noscript", "template", "iframe", "frame", "frameset", "object", "embed", "applet", "param", "svg",
    "math", "audio", "video", "source", "track", "canvas", "map", "area", "link", "meta", "base", "title", "button",
    "input", "select", "option", "optgroup", "datalist", "textarea", "dialog", "portal", "slot", "xmp", "noembed",
    "noframes", "plaintext", "bgsound", "isindex", "keygen",
];

/// Elements with no end tag.
const VOID: &[&str] = &["br", "col", "hr", "img", "wbr"];

pub(super) fn policy(local: &str) -> Policy {
    if local == "style" {
        Policy::Style
    } else if KEPT.contains(&local) {
        Policy::Keep
    } else if DROPPED.contains(&local) {
        Policy::Drop
    } else {
        Policy::Unwrap
    }
}

pub(super) fn is_void(local: &str) -> bool {
    VOID.contains(&local)
}

/// Attributes that are kept on every kept element. Their values are written
/// escaped; none of them can load anything or run anything.
const EVERYWHERE: &[&str] = &[
    "align", "bgcolor", "border", "cellpadding", "cellspacing", "class", "color", "colspan", "dir", "face", "headers",
    "height", "id", "lang", "nowrap", "rowspan", "scope", "size", "title", "valign", "width",
];

/// Attributes kept only on one element.
const PER_ELEMENT: &[(&str, &str)] = &[
    ("a", "href"),
    ("img", "alt"),
    ("img", "src"),
    ("ol", "start"),
    ("ol", "type"),
    ("ol", "reversed"),
    ("li", "value"),
    ("time", "datetime"),
    ("ins", "datetime"),
    ("del", "datetime"),
    ("td", "abbr"),
    ("th", "abbr"),
];

/// Whether `attribute` may be written on a kept `element`. The `style`
/// attribute is not in this table: it is filtered and written separately.
pub(super) fn attribute_kept(element: &str, attribute: &str) -> bool {
    EVERYWHERE.contains(&attribute) || PER_ELEMENT.contains(&(element, attribute))
}

/// Attributes that name something to load. A reference to the network in one
/// of these is counted when it is left out.
pub(super) const REFERENCES: &[&str] =
    &["src", "href", "data", "poster", "background", "srcset", "action", "formaction", "longdesc", "lowsrc", "dynsrc"];

#[cfg(test)]
include!("allow.tests.rs");
