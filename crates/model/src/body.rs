//! A message body as styled lines, plus its attachment list and the
//! address its answers should go to.

use crate::address::Address;

/// What a run of body text is; each role maps to a `body.*` theme role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BodyRole {
    #[default]
    Text,
    Quote,
    /// The display name inside a body.
    Name,
    BulletDone,
    BulletOpen,
    Lead,
    LeadNext,
    Token,
    Path,
    Link,
    Signature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledSpan {
    pub text: String,
    pub role: BodyRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StyledLine {
    pub spans: Vec<StyledSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentInfo {
    /// Index of the part in the message's part tree.
    pub part: u32,
    /// The number the server gives the part inside the message ("2", "1.3"),
    /// which is how the part is asked for. Empty when the message has no
    /// server to ask (the mock).
    pub section: String,
    pub name: String,
    pub content_type: String,
    /// Size in bytes.
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Body {
    pub lines: Vec<StyledLine>,
    pub attachments: Vec<AttachmentInfo>,
    /// Where the sender asks answers to go (the Reply-To header); empty when
    /// the message has none.
    pub reply_to: Vec<Address>,
}
