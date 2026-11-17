//! The application's name, tagline, version and home page, handed to the
//! screen code by its caller. The screen code never spells any of them.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppIdentity {
    /// Display name, as shown in the banner and the footer.
    pub name: String,
    /// Short description shown beside the name in the banner.
    pub tagline: String,
    /// Version number without a leading `v`.
    pub version: String,
    /// The address of the home page; empty when the caller has none.
    pub homepage: String,
}

impl AppIdentity {
    pub fn new(
        name: impl Into<String>,
        tagline: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            tagline: tagline.into(),
            version: version.into(),
            homepage: String::new(),
        }
    }

    /// The same identity with its home page set.
    pub fn with_homepage(mut self, homepage: impl Into<String>) -> Self {
        self.homepage = homepage.into();
        self
    }

    /// The version as the screen writes it: `v1.2.3`.
    pub fn version_tag(&self) -> String {
        format!("v{}", self.version)
    }

    /// The name and the version tag: `Name v1.2.3`.
    pub fn name_and_version(&self) -> String {
        format!("{} {}", self.name, self.version_tag())
    }
}

#[cfg(test)]
include!("app_identity.tests.rs");
