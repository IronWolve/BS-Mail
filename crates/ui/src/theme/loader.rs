//! Reads a theme file: parses the text, follows `inherits` and checks that
//! every color a role names exists, with errors that name the role at fault.

use std::fmt;

use model::theme_schema::{BASE_ROLE, ColorRef, Palette, Rgb, Role, ThemeFile};

/// Why a theme file could not be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeError {
    /// The text is not valid TOML or does not match the theme schema; the
    /// message carries the line and the offending value.
    Parse(String),
    /// A role names a color that is neither in the palette nor `#rrggbb`.
    UnknownColor {
        role: String,
        field: &'static str,
        name: String,
    },
    /// The theme has no `base` role.
    MissingBaseRole,
    /// The `base` role has no foreground color.
    BaseWithoutForeground,
    /// A role writes `gradient = []`.
    EmptyGradient { role: String },
    /// `inherits` names a theme the lookup does not know.
    UnknownParent { theme: String, parent: String },
    /// A chain of `inherits` leads back to a theme already in the chain.
    InheritCycle { parent: String },
}

impl fmt::Display for ThemeError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeError::Parse(message) => write!(out, "theme file: {message}"),
            ThemeError::UnknownColor { role, field, name } => {
                write!(out, "role `{role}`: `{field}` names unknown color `{name}`")
            }
            ThemeError::MissingBaseRole => write!(out, "theme has no `{BASE_ROLE}` role"),
            ThemeError::BaseWithoutForeground => {
                write!(out, "role `{BASE_ROLE}` must set `fg`")
            }
            ThemeError::EmptyGradient { role } => {
                write!(out, "role `{role}`: `gradient` needs at least one color")
            }
            ThemeError::UnknownParent { theme, parent } => {
                write!(out, "theme `{theme}` inherits unknown theme `{parent}`")
            }
            ThemeError::InheritCycle { parent } => {
                write!(out, "theme inheritance loops back to `{parent}`")
            }
        }
    }
}

impl std::error::Error for ThemeError {}

/// Parses theme text into the schema type, with no inheritance or checks.
pub fn parse_theme_file(text: &str) -> Result<ThemeFile, ThemeError> {
    toml::from_str(text).map_err(|err| ThemeError::Parse(err.to_string()))
}

/// Parses theme text, follows `inherits` through `find` (theme name to theme
/// text), merges the chain into one flat file and validates the result.
pub fn load_theme_file(
    text: &str,
    find: &dyn Fn(&str) -> Option<String>,
) -> Result<ThemeFile, ThemeError> {
    let merged = load_chain(text, find, &mut Vec::new())?;
    validate(&merged)?;
    Ok(merged)
}

fn load_chain(
    text: &str,
    find: &dyn Fn(&str) -> Option<String>,
    requested: &mut Vec<String>,
) -> Result<ThemeFile, ThemeError> {
    let file = parse_theme_file(text)?;
    let Some(parent_name) = file.inherits.clone() else {
        return Ok(file);
    };
    if requested.contains(&parent_name) {
        return Err(ThemeError::InheritCycle {
            parent: parent_name,
        });
    }
    requested.push(parent_name.clone());
    let parent_text = find(&parent_name).ok_or_else(|| ThemeError::UnknownParent {
        theme: file.name.clone(),
        parent: parent_name,
    })?;
    let parent = load_chain(&parent_text, find, requested)?;
    Ok(merge_inherited(parent, file))
}

/// Lays `child` over `parent`: palette entries and `ansi16` entries of the
/// child replace the parent's, and a role present in both is merged field by
/// field, the child's fields winning. The result no longer inherits.
pub fn merge_inherited(parent: ThemeFile, child: ThemeFile) -> ThemeFile {
    let mut merged = parent;
    if !child.name.is_empty() {
        merged.name = child.name;
    }
    merged.inherits = None;
    merged.palette.colors.extend(child.palette.colors);
    merged.ansi16.extend(child.ansi16);
    for (name, role) in child.roles {
        let combined = match merged.roles.remove(&name) {
            Some(old) => overlay_role(old, role),
            None => role,
        };
        merged.roles.insert(name, combined);
    }
    merged
}

fn overlay_role(old: Role, new: Role) -> Role {
    Role {
        fg: new.fg.or(old.fg),
        bg: new.bg.or(old.bg),
        gradient: new.gradient.or(old.gradient),
        bold: new.bold.or(old.bold),
        italic: new.italic.or(old.italic),
        dim: new.dim.or(old.dim),
        underline: new.underline.or(old.underline),
    }
}

/// Checks a flat theme: a `base` role with a foreground, and every color any
/// role names resolves. Reports the first problem, naming the role.
pub fn validate(file: &ThemeFile) -> Result<(), ThemeError> {
    let base = file.roles.get(BASE_ROLE).ok_or(ThemeError::MissingBaseRole)?;
    if base.fg.is_none() && base.gradient.is_none() {
        return Err(ThemeError::BaseWithoutForeground);
    }
    for (name, role) in &file.roles {
        check_role(&file.palette, name, role)?;
    }
    Ok(())
}

fn check_role(palette: &Palette, name: &str, role: &Role) -> Result<(), ThemeError> {
    for (field, color) in [("fg", &role.fg), ("bg", &role.bg)] {
        if let Some(color) = color {
            color_of(palette, name, field, color)?;
        }
    }
    if let Some(stops) = &role.gradient {
        if stops.is_empty() {
            return Err(ThemeError::EmptyGradient {
                role: name.to_string(),
            });
        }
        for stop in stops {
            color_of(palette, name, "gradient", stop)?;
        }
    }
    Ok(())
}

/// Looks a color reference up in the palette; a `#rrggbb` value passes
/// through. `role` and `field` only label the error.
pub fn color_of(
    palette: &Palette,
    role: &str,
    field: &'static str,
    color: &ColorRef,
) -> Result<Rgb, ThemeError> {
    match color {
        ColorRef::Hex(rgb) => Ok(*rgb),
        ColorRef::Named(name) => palette.get(name).ok_or_else(|| ThemeError::UnknownColor {
            role: role.to_string(),
            field,
            name: name.clone(),
        }),
    }
}

#[cfg(test)]
include!("loader.tests.rs");
