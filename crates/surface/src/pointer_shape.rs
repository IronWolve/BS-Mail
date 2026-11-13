//! The pointer shapes the UI can ask a host to show.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PointerShape {
    #[default]
    Arrow,
    Hand,
    Text,
    ColumnResize,
    RowResize,
}
