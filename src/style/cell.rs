/// This can be set on [columns](crate::Column::set_cell_alignment) and [cells](crate::Cell::set_alignment).
///
/// Determines how content of cells should be aligned horizontally.
///
/// ```text
/// +----------------------+
/// | Header1              |
/// +======================+
/// | Left                 |
/// |----------------------+
/// |        center        |
/// |----------------------+
/// |                right |
/// +----------------------+
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CellAlignment {
    Left,
    Right,
    Center,
}

/// Determines how content of cells should be aligned vertically.
///
/// This is useful when cells in the same row have different heights
/// (e.g., one cell has multi-line content while another has single-line content).
///
/// ```text
/// +--------+-----+        +--------+-----+        +--------+-----+
/// | Line 1 | Top |        | Line 1 |     |        | Line 1 |     |
/// | Line 2 |     |        | Line 2 | Mid |        | Line 2 |     |
/// | Line 3 |     |        | Line 3 |     |        | Line 3 | Bot |
/// +--------+-----+        +--------+-----+        +--------+-----+
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum VerticalAlignment {
    /// Content is aligned to the top of the cell (default)
    #[default]
    Top,
    /// Content is centered vertically in the cell
    Middle,
    /// Content is aligned to the bottom of the cell
    Bottom,
}
