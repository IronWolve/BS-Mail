//! Which highlight rules are on.

/// One switch per rule; every rule is on by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighlightRules {
    /// `✓`, `[x]`, `- [x]` and `○`, `[ ]`, `- [ ]` at the start of a line.
    pub bullets: bool,
    /// A `Word:` after a bullet.
    pub lead: bool,
    /// `keyword:` and `:word` search syntax.
    pub tokens: bool,
    /// `v1.3.0` and `1.3.0`.
    pub versions: bool,
    /// `(path/like-token)`.
    pub paths: bool,
    /// `— Name` and `-- Name` at the start of a line.
    pub signature: bool,
    /// Lines starting with `>`.
    pub quote: bool,
    /// `http://` and `https://` addresses.
    pub links: bool,
    /// The application's display name.
    pub name: bool,
}

impl Default for HighlightRules {
    fn default() -> Self {
        Self {
            bullets: true,
            lead: true,
            tokens: true,
            versions: true,
            paths: true,
            signature: true,
            quote: true,
            links: true,
            name: true,
        }
    }
}
