//! The bytes of a file as they travel inside commands and events: a file to
//! send, or an attachment that was fetched. They are shared, so cloning a
//! command or an event never copies the file, and they are never printed: the
//! debug output shows a size only, so a log or a failed test cannot dump a
//! file's contents.

use std::fmt;
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq, Default)]
pub struct FileBytes(Arc<Vec<u8>>);

impl FileBytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(Arc::new(bytes))
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for FileBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileBytes({} bytes)", self.0.len())
    }
}

#[cfg(test)]
include!("file_bytes.tests.rs");
