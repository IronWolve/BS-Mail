//! The whole theme file: name, parent, palette, roles and the 16-color table.
//!
//! Role names contain dots (`border.focused`), so in the file each one is a
//! quoted key: `"border.focused" = { fg = "green" }`. A bare dotted key would
//! read as nested tables and could not hold both a role and its children.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::palette::{Ansi16Color, Palette};
use super::role::Role;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeFile {
    pub name: String,
    /// Name of a theme to start from; this file overrides only what differs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherits: Option<String>,
    pub palette: Palette,
    /// Role name (dotted) to role.
    pub roles: BTreeMap<String, Role>,
    /// Palette color name to terminal color, for 16-color terminals only.
    pub ansi16: BTreeMap<String, Ansi16Color>,
}
