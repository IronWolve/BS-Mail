//! A loaded theme: every role resolved once, with inheritance from the parent
//! dotted name and from `base`, and a record of roles asked for but missing.

use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::sync::{Mutex, PoisonError};

use model::theme_schema::{Ansi16Color, BASE_ROLE, Palette, Rgb, Role, ThemeFile};

use super::loader::{ThemeError, color_of};
use super::resolved_style::ResolvedStyle;

/// One role with its colors looked up but its inheritance not yet applied.
struct PartialRole {
    fg: Option<Rgb>,
    bg: Option<Rgb>,
    gradient: Option<Vec<Rgb>>,
    bold: Option<bool>,
    italic: Option<bool>,
    dim: Option<bool>,
    underline: Option<bool>,
}

#[derive(Debug)]
pub struct Theme {
    name: String,
    palette: BTreeMap<String, Rgb>,
    ansi16: BTreeMap<String, Ansi16Color>,
    base: ResolvedStyle,
    styles: BTreeMap<String, ResolvedStyle>,
    unknown: Mutex<BTreeSet<String>>,
}

impl Theme {
    /// Builds a theme from a flat theme file (see `load_theme_file`).
    pub fn from_file(file: &ThemeFile) -> Result<Self, ThemeError> {
        let mut partials = BTreeMap::new();
        for (name, role) in &file.roles {
            partials.insert(name.clone(), look_up(&file.palette, name, role)?);
        }
        let base_fg = partials
            .get(BASE_ROLE)
            .ok_or(ThemeError::MissingBaseRole)?
            .fg_or_first_stop()
            .ok_or(ThemeError::BaseWithoutForeground)?;
        let styles: BTreeMap<String, ResolvedStyle> = partials
            .keys()
            .map(|name| (name.clone(), resolve_role(&partials, name, base_fg)))
            .collect();
        let base = resolve_role(&partials, BASE_ROLE, base_fg);
        Ok(Self {
            name: file.name.clone(),
            palette: file.palette.colors.clone(),
            ansi16: file.ansi16.clone(),
            base,
            styles,
            unknown: Mutex::new(BTreeSet::new()),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// The style of `name`. A role the theme does not define returns the base
    /// style and is recorded once in `unknown_roles`.
    pub fn role(&self, name: &str) -> ResolvedStyle {
        match self.styles.get(name) {
            Some(style) => style.clone(),
            None => {
                self.unknown
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .insert(name.to_string());
                self.base.clone()
            }
        }
    }

    pub fn has_role(&self, name: &str) -> bool {
        self.styles.contains_key(name)
    }

    /// Every role the theme defines, in name order.
    pub fn role_names(&self) -> impl Iterator<Item = &str> {
        self.styles.keys().map(String::as_str)
    }

    /// The window background: the background of `base`.
    pub fn background(&self) -> Option<Rgb> {
        self.base.bg
    }

    pub fn palette_color(&self, name: &str) -> Option<Rgb> {
        self.palette.get(name).copied()
    }

    /// The theme's `[ansi16]` table: palette color name to terminal color.
    pub fn ansi16(&self) -> &BTreeMap<String, Ansi16Color> {
        &self.ansi16
    }

    /// Names passed to `role` that the theme does not define, sorted, each once.
    pub fn unknown_roles(&self) -> Vec<String> {
        let unknown = self.unknown.lock().unwrap_or_else(PoisonError::into_inner);
        unknown.iter().cloned().collect()
    }
}

impl PartialRole {
    fn fg_or_first_stop(&self) -> Option<Rgb> {
        self.fg
            .or_else(|| self.gradient.as_ref().and_then(|stops| stops.first().copied()))
    }
}

fn look_up(palette: &Palette, name: &str, role: &Role) -> Result<PartialRole, ThemeError> {
    let single = |field: &'static str, color: &Option<_>| match color {
        Some(color) => color_of(palette, name, field, color).map(Some),
        None => Ok(None),
    };
    let gradient = match &role.gradient {
        None => None,
        Some(stops) if stops.is_empty() => {
            return Err(ThemeError::EmptyGradient {
                role: name.to_string(),
            });
        }
        Some(stops) => Some(
            stops
                .iter()
                .map(|stop| color_of(palette, name, "gradient", stop))
                .collect::<Result<Vec<_>, _>>()?,
        ),
    };
    Ok(PartialRole {
        fg: single("fg", &role.fg)?,
        bg: single("bg", &role.bg)?,
        gradient,
        bold: role.bold,
        italic: role.italic,
        dim: role.dim,
        underline: role.underline,
    })
}

/// Applies inheritance: each field comes from the role itself, else from its
/// parent dotted name (`list.date.recent` -> `list.date` -> `list`), else
/// from `base`. Parents the theme does not define are skipped.
fn resolve_role(
    partials: &BTreeMap<String, PartialRole>,
    name: &str,
    base_fg: Rgb,
) -> ResolvedStyle {
    let chain: Vec<&PartialRole> =
        iter::successors(Some(name), |n| n.rsplit_once('.').map(|(parent, _)| parent))
            .chain(iter::once(BASE_ROLE))
            .filter_map(|n| partials.get(n))
            .collect();
    let flag = |get: fn(&PartialRole) -> Option<bool>| {
        chain.iter().find_map(|p| get(p)).unwrap_or(false)
    };
    // The nearest role that sets a color decides it; a gradient there wins
    // over a plain foreground, and a nearer plain foreground hides a
    // gradient further up.
    let (fg, gradient) = match chain
        .iter()
        .find(|p| p.fg.is_some() || p.gradient.is_some())
    {
        Some(p) => match &p.gradient {
            Some(stops) => (stops.first().copied().unwrap_or(base_fg), Some(stops.clone())),
            None => (p.fg.unwrap_or(base_fg), None),
        },
        None => (base_fg, None),
    };
    ResolvedStyle {
        fg,
        bg: chain.iter().find_map(|p| p.bg),
        gradient,
        bold: flag(|p| p.bold),
        italic: flag(|p| p.italic),
        dim: flag(|p| p.dim),
        underline: flag(|p| p.underline),
    }
}

#[cfg(test)]
include!("resolve.tests.rs");
