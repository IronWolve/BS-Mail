//! The title banner. Two rows: slashes, a double-size icon and name with the
//! theme gradient, a divider, the tagline, then the version tag and slashes
//! at the right, all half-row shifted. One row: the same at normal size.
//! The cells under the double-size text hold the name at normal size, so a
//! host without effects still shows it.

use model::theme_schema::Rgb;
use surface::{Color, Effect, Frame, Glow, HalfRowShift, Rect, ScaledText, Style, TextFill};

use super::cell_text::{Run, cells, put_cut, put_runs, put_text};
use super::draw_context::DrawContext;
use crate::app_identity::AppIdentity;
use crate::icons::IconId;
use crate::layout::inset_h;
use crate::theme::{roles, spread, surface_color};
use unicode_segmentation::UnicodeSegmentation;

/// Blank cells after the left slashes, before the icon.
const SLASH_GAP: u16 = 2;
/// Blank cells between the version tag and the right slashes.
const VERSION_GAP: u16 = 2;
/// Blank cells on each side of the divider bar.
const BAR_GAP: u16 = 2;
/// The tagline is left out when fewer cells than this remain for it.
const TAGLINE_MIN: u16 = 4;
/// Glow radius in pixels at scale 1.
const GLOW_RADIUS: u8 = 6;

/// Where each part of the banner goes, in columns.
#[derive(Debug, Clone, PartialEq, Eq)]
struct BannerPlan {
    double: bool,
    left_slashes: u16,
    icon_x: u16,
    icon_width: u16,
    name_x: u16,
    name_width: u16,
    /// Divider column and tagline start, when the tagline fits.
    tagline: Option<(u16, u16, u16)>,
    right: Option<RightGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RightGroup {
    x: u16,
    width: u16,
    version: Option<(u16, String)>,
    slashes: Option<(u16, u16)>,
}

/// What the banner shows, resolved from the identity and the options.
struct BannerSpec {
    content: Rect,
    double: bool,
    slashes: u16,
    name: String,
    tagline: String,
    version: Option<String>,
}

fn plan_banner(spec: &BannerSpec) -> BannerPlan {
    let scale = if spec.double { 2 } else { 1 };
    let mut x = spec.content.x;
    if spec.slashes > 0 {
        x += spec.slashes + SLASH_GAP;
    }
    let icon_width = 2 * scale;
    let icon_x = x;
    let name_x = icon_x.saturating_add(icon_width + 1);
    let name_width = cells(&spec.name).saturating_mul(scale);
    let name_end = name_x.saturating_add(name_width);
    let right = plan_right(spec, name_end);
    let limit = right.as_ref().map_or(spec.content.right(), |group| group.x.saturating_sub(1));
    let tag_x = name_end.saturating_add(1 + 2 * BAR_GAP);
    let tag_room = limit.saturating_sub(tag_x);
    let tagline = (tag_room >= TAGLINE_MIN && !spec.tagline.is_empty())
        .then_some((name_end.saturating_add(BAR_GAP), tag_x, tag_room));
    BannerPlan {
        double: spec.double,
        left_slashes: spec.slashes,
        icon_x,
        icon_width,
        name_x,
        name_width,
        tagline,
        right,
    }
}

/// The version tag and slashes, right-aligned, or none when they would
/// crowd the name.
fn plan_right(spec: &BannerSpec, name_end: u16) -> Option<RightGroup> {
    let version_width = spec.version.as_ref().map_or(0, |v| cells(v));
    let mut width = version_width + spec.slashes;
    if version_width > 0 && spec.slashes > 0 {
        width += VERSION_GAP;
    }
    if width == 0 {
        return None;
    }
    let x = spec.content.right().checked_sub(width)?;
    if x <= name_end {
        return None;
    }
    let version = spec.version.clone().map(|text| (x, text));
    let slashes_x = x + width - spec.slashes;
    Some(RightGroup {
        x,
        width,
        version,
        slashes: (spec.slashes > 0).then_some((slashes_x, spec.slashes)),
    })
}

/// Draws the banner into `rect` (one or two rows, inside the frame).
pub fn draw_banner(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, identity: &AppIdentity) {
    if rect.is_empty() {
        return;
    }
    let banner = &ctx.config.banner;
    let tagline = if banner.tagline.is_empty() { &identity.tagline } else { &banner.tagline };
    let spec = BannerSpec {
        content: inset_h(rect, 1),
        double: rect.height >= 2,
        slashes: if banner.slashes { u16::from(banner.slash_count.min(12)) } else { 0 },
        name: identity.name.clone(),
        tagline: tagline.clone(),
        version: banner.show_version.then(|| identity.version_tag()),
    };
    let plan = plan_banner(&spec);
    let clip = spec.content;
    draw_left(frame, ctx, &plan, clip, (rect.y, &spec.name));
    if let Some((bar_x, tag_x, room)) = plan.tagline {
        draw_tagline(frame, ctx, &plan, (clip, rect), (bar_x, tag_x, room), &spec.tagline);
    }
    if let Some(group) = &plan.right {
        draw_right(frame, ctx, &plan, clip, rect.y, group);
    }
}

fn shift(frame: &mut Frame, plan: &BannerPlan, area: Rect) {
    if plan.double {
        frame.push_effect(Effect::HalfRowShift(HalfRowShift { area }));
    }
}

fn draw_left(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    plan: &BannerPlan,
    clip: Rect,
    (y, name): (u16, &str),
) {
    if plan.left_slashes > 0 {
        let slashes = "/".repeat(usize::from(plan.left_slashes));
        put_text(frame, clip, clip.x, y, &slashes, ctx.fg(roles::BANNER_SLASHES));
        shift(frame, plan, Rect::new(clip.x, y, plan.left_slashes, 1));
    }
    put_text(frame, clip, plan.icon_x, y, ctx.icon(IconId::Mail), ctx.fg(roles::BANNER_ICON));
    let stops = name_stops(ctx);
    put_runs(frame, clip, plan.name_x, y, &name_runs(ctx, &stops, name));
    if plan.double {
        scaled_icon(frame, ctx, plan, clip, y);
        scaled_name(frame, ctx, plan, clip, (y, name, &stops));
    }
}

fn scaled_icon(frame: &mut Frame, ctx: &DrawContext<'_>, plan: &BannerPlan, clip: Rect, y: u16) {
    let area = Rect::new(plan.icon_x, y, plan.icon_width, 2);
    if area.right() > clip.right() {
        return;
    }
    frame.push_effect(Effect::ScaledText(ScaledText {
        area,
        text: ctx.icon_inline(IconId::Mail).to_string(),
        scale: 2,
        fill: TextFill::Solid(ctx.fg_color(roles::BANNER_ICON)),
        glow: None,
        bold: false,
        italic: false,
    }));
}

/// The stops of the name gradient, or the one plain color when the gradient
/// is switched off or the theme gives none.
fn name_stops(ctx: &DrawContext<'_>) -> Vec<Rgb> {
    let role = ctx.theme.role(roles::BANNER_NAME);
    match (role.gradient, ctx.config.banner.gradient) {
        (Some(stops), true) => stops,
        _ => vec![role.fg],
    }
}

/// The name at normal size, one run per character with its gradient color.
fn name_runs(ctx: &DrawContext<'_>, stops: &[Rgb], name: &str) -> Vec<Run> {
    let weight = ctx.theme.role(roles::BANNER_NAME).modifiers();
    let parts: Vec<&str> = name.graphemes(true).collect();
    let colors = spread(stops, parts.len());
    parts
        .iter()
        .zip(&colors)
        .map(|(part, rgb)| {
            Run::new(*part, Style::new().fg(surface_color(*rgb)).add_modifier(weight))
        })
        .collect()
}

fn scaled_name(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    plan: &BannerPlan,
    clip: Rect,
    (y, name, stops): (u16, &str, &[Rgb]),
) {
    let area = Rect::new(plan.name_x, y, plan.name_width, 2);
    if area.right() > clip.right() {
        return;
    }
    let colors: Vec<Color> = stops.iter().map(|rgb| surface_color(*rgb)).collect();
    let role = ctx.theme.role(roles::BANNER_NAME);
    let glow = ctx.config.banner.glow.then(|| Glow {
        color: colors[0],
        radius: GLOW_RADIUS,
    });
    let fill = if colors.len() > 1 {
        TextFill::Gradient(colors)
    } else {
        TextFill::Solid(colors[0])
    };
    frame.push_effect(Effect::ScaledText(ScaledText {
        area,
        text: name.to_string(),
        scale: 2,
        fill,
        glow,
        bold: role.bold,
        italic: role.italic,
    }));
}

fn draw_tagline(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    plan: &BannerPlan,
    (clip, rect): (Rect, Rect),
    (bar_x, tag_x, room): (u16, u16, u16),
    tagline: &str,
) {
    let bar = ctx.borders.vertical.to_string();
    for dy in 0..rect.height {
        put_text(frame, clip, bar_x, rect.y + dy, &bar, ctx.fg(roles::BANNER_DIVIDER));
    }
    let end = put_cut(frame, clip, tag_x, rect.y, tagline, room, ctx.fg(roles::BANNER_TAGLINE));
    shift(frame, plan, Rect::new(tag_x, rect.y, end.saturating_sub(tag_x), 1));
}

fn draw_right(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    plan: &BannerPlan,
    clip: Rect,
    y: u16,
    group: &RightGroup,
) {
    if let Some((x, text)) = &group.version {
        put_text(frame, clip, *x, y, text, ctx.fg(roles::BANNER_VERSION));
    }
    if let Some((x, count)) = group.slashes {
        let slashes = "/".repeat(usize::from(count));
        put_text(frame, clip, x, y, &slashes, ctx.fg(roles::BANNER_SLASHES));
    }
    shift(frame, plan, Rect::new(group.x, y, group.width, 1));
}

#[cfg(test)]
include!("banner.tests.rs");
