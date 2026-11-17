//! The links of the body as a list in reading order, for stepping from one to
//! the next with the keys. It is made from the same wrapped rows the drawing,
//! the click and the light use, so a link here is the link they all mean: its
//! place among the body's links is the number its pieces carry.

use surface::Rect;

use super::draw::{BodyView, wrapped};
use crate::widgets::DrawContext;

/// One link of the body, with where it shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkSpan {
    /// Its place among the body's links (see `LinkHit`).
    pub index: usize,
    /// The whole address, whatever the rows it is cut over.
    pub address: String,
    /// The first and the last wrapped row of the body it is on.
    pub rows: (usize, usize),
}

/// The links of the body drawn in `rect` with `view`, in the order they come
/// in the body. A link that has no cell on any row (one made only of
/// characters that take no room) is not listed: there is nothing to light.
pub fn body_links(ctx: &DrawContext<'_>, rect: Rect, view: &BodyView<'_>) -> Vec<LinkSpan> {
    let body = wrapped(ctx, rect, view);
    let mut rows: Vec<Option<(usize, usize)>> = vec![None; body.links.len()];
    for (row, pieces) in body.rows.iter().enumerate() {
        for piece in pieces {
            if let Some(slot) = piece.link.and_then(|link| rows.get_mut(link)) {
                slot.get_or_insert((row, row)).1 = row;
            }
        }
    }
    rows.into_iter()
        .zip(&body.links)
        .enumerate()
        .filter_map(|(index, (rows, address))| Some(LinkSpan { index, address: address.clone(), rows: rows? }))
        .collect()
}

#[cfg(test)]
include!("link_list.tests.rs");
