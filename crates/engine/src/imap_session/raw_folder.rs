//! One mailbox as the server listed it, before roles and tree are worked out.

/// A LIST entry plus the counts from STATUS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFolder {
    /// The full name exactly as the server sent it; used in commands.
    pub path: String,
    /// The hierarchy delimiter; `None` for a flat namespace.
    pub delimiter: Option<String>,
    /// Attributes in lower case with their backslash, such as `\sent`.
    pub attributes: Vec<String>,
    pub unread: u32,
    pub total: u32,
}

impl RawFolder {
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.iter().any(|a| a == name)
    }

    /// The name of the folder this one sits in, when the path has a parent.
    pub fn parent_path(&self) -> Option<&str> {
        let delimiter = self.delimiter.as_deref().filter(|d| !d.is_empty())?;
        self.path.rfind(delimiter).map(|at| &self.path[..at])
    }

    /// The last path segment, still in the server's encoding.
    pub fn leaf(&self) -> &str {
        match self.delimiter.as_deref().filter(|d| !d.is_empty()) {
            Some(delimiter) => self
                .path
                .rsplit(delimiter)
                .next()
                .unwrap_or(&self.path),
            None => &self.path,
        }
    }
}

#[cfg(test)]
include!("raw_folder.tests.rs");
