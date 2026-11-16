//! The ceilings of the safe copy. A message can be made as large or as
//! tangled as its sender likes, so each step has one.

/// The most HTML read, as for the text converter (see `mime_render::limits`).
pub(super) const MAX_HTML_BYTES: usize = 2 * 1024 * 1024;
/// Elements nested deeper than this are not written.
pub(super) const MAX_DEPTH: usize = 256;
/// The most the page's body may grow to.
pub(super) const MAX_BODY_BYTES: usize = 6 * 1024 * 1024;
/// The most of one style sheet that is read, and of all of them that is kept.
pub(super) const MAX_SHEET_BYTES: usize = 256 * 1024;
pub(super) const MAX_KEPT_CSS_BYTES: usize = 128 * 1024;
/// The largest picture that is put into the page, in bytes of the picture.
pub(super) const MAX_PICTURE_BYTES: usize = 256 * 1024;
/// The most all the pictures of a page add up to.
pub(super) const MAX_PICTURES_TOTAL: usize = 4 * 1024 * 1024;
/// The longest address a link may have.
pub(super) const MAX_LINK_BYTES: usize = 8 * 1024;
