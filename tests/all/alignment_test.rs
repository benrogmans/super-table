use pretty_assertions::assert_eq;

use super_table::*;

#[test]
/// Cell alignment can be specified on Columns and Cells
/// Alignment settings on Cells overwrite the settings of Columns
fn cell_alignment() {
    let mut table = Table::new();
    table
        .set_header(vec!["Header1", "Header2", "Header3"])
        .add_row(vec![
            "Very long line Test",
            "Very long line Test",
            "Very long line Test",
        ])
        .add_row(vec![
            Cell::new("Right").set_alignment(CellAlignment::Right),
            Cell::new("Left").set_alignment(CellAlignment::Left),
            Cell::new("Center").set_alignment(CellAlignment::Center),
        ])
        .add_row(vec!["Left", "Center", "Right"]);

    let alignment = [
        CellAlignment::Left,
        CellAlignment::Center,
        CellAlignment::Right,
    ];

    // Add the alignment to their respective column
    for (column_index, column) in table.column_iter_mut().enumerate() {
        let alignment = alignment.get(column_index).unwrap();
        column.set_cell_alignment(*alignment);
    }

    println!("{table}");
    let expected = "
+---------------------+---------------------+---------------------+
| Header1             |       Header2       |             Header3 |
+=================================================================+
| Very long line Test | Very long line Test | Very long line Test |
|---------------------+---------------------+---------------------|
|               Right | Left                |        Center       |
|---------------------+---------------------+---------------------|
| Left                |        Center       |               Right |
+---------------------+---------------------+---------------------+";
    assert_eq!(expected, "\n".to_string() + &table.to_string());
}

#[test]
fn vertical_alignment_top() {
    let mut table = Table::new();
    table.set_header(vec!["H1", "H2"]).add_row(vec![
        Cell::new("Line 1\nLine 2\nLine 3"),
        Cell::new("Top").set_vertical_alignment(VerticalAlignment::Top),
    ]);

    let expected = "
+--------+-----+
| H1     | H2  |
+==============+
| Line 1 | Top |
| Line 2 |     |
| Line 3 |     |
+--------+-----+";
    assert_eq!(expected, "\n".to_string() + &table.to_string());
}

#[test]
fn vertical_alignment_middle() {
    let mut table = Table::new();
    table.set_header(vec!["H1", "H2"]).add_row(vec![
        Cell::new("Line 1\nLine 2\nLine 3"),
        Cell::new("Mid").set_vertical_alignment(VerticalAlignment::Middle),
    ]);

    let expected = "
+--------+-----+
| H1     | H2  |
+==============+
| Line 1 |     |
| Line 2 | Mid |
| Line 3 |     |
+--------+-----+";
    assert_eq!(expected, "\n".to_string() + &table.to_string());
}

#[test]
fn vertical_alignment_bottom() {
    let mut table = Table::new();
    table.set_header(vec!["H1", "H2"]).add_row(vec![
        Cell::new("Line 1\nLine 2\nLine 3"),
        Cell::new("Bot").set_vertical_alignment(VerticalAlignment::Bottom),
    ]);

    let expected = "
+--------+-----+
| H1     | H2  |
+==============+
| Line 1 |     |
| Line 2 |     |
| Line 3 | Bot |
+--------+-----+";
    assert_eq!(expected, "\n".to_string() + &table.to_string());
}

#[test]
fn vertical_alignment_with_multiline_content() {
    let mut table = Table::new();
    table.set_header(vec!["H1", "H2", "H3"]).add_row(vec![
        Cell::new("1\n2\n3\n4\n5"),
        Cell::new("Top\nTwo").set_vertical_alignment(VerticalAlignment::Top),
        Cell::new("Bot\nTwo").set_vertical_alignment(VerticalAlignment::Bottom),
    ]);

    let expected = "
+----+-----+-----+
| H1 | H2  | H3  |
+================+
| 1  | Top |     |
| 2  | Two |     |
| 3  |     |     |
| 4  |     | Bot |
| 5  |     | Two |
+----+-----+-----+";
    assert_eq!(expected, "\n".to_string() + &table.to_string());
}

#[test]
fn vertical_alignment_combined_with_horizontal() {
    let mut table = Table::new();
    table.set_header(vec!["H1", "H2"]).add_row(vec![
        Cell::new("Line 1\nLine 2\nLine 3"),
        Cell::new("RC")
            .set_alignment(CellAlignment::Right)
            .set_vertical_alignment(VerticalAlignment::Middle),
    ]);

    let expected = "
+--------+----+
| H1     | H2 |
+=============+
| Line 1 |    |
| Line 2 | RC |
| Line 3 |    |
+--------+----+";
    assert_eq!(expected, "\n".to_string() + &table.to_string());
}

#[test]
fn vertical_alignment_with_rowspan() {
    let mut table = Table::new();
    table
        .set_header(vec!["H1", "H2", "H3"])
        .add_row(vec![
            Cell::new("Centered")
                .set_rowspan(3)
                .set_vertical_alignment(VerticalAlignment::Middle),
            Cell::new("Row 1 Col 2"),
            Cell::new("Row 1 Col 3"),
        ])
        .add_row(vec![Cell::new("Row 2 Col 2"), Cell::new("Row 2 Col 3")])
        .add_row(vec![Cell::new("Row 3 Col 2"), Cell::new("Row 3 Col 3")]);

    let expected = "
+----------+-------------+-------------+
| H1       | H2          | H3          |
+======================================+
|          | Row 1 Col 2 | Row 1 Col 3 |
|          |-------------+-------------|
| Centered | Row 2 Col 2 | Row 2 Col 3 |
|          |-------------+-------------|
|          | Row 3 Col 2 | Row 3 Col 3 |
+----------+-------------+-------------+";
    assert_eq!(expected, "\n".to_string() + &table.to_string());
}

#[test]
fn vertical_alignment_default_is_top() {
    let mut table = Table::new();
    table.set_header(vec!["H1", "H2"]).add_row(vec![
        Cell::new("Line 1\nLine 2\nLine 3"),
        Cell::new("Default"),
    ]);

    let expected = "
+--------+---------+
| H1     | H2      |
+==================+
| Line 1 | Default |
| Line 2 |         |
| Line 3 |         |
+--------+---------+";
    assert_eq!(expected, "\n".to_string() + &table.to_string());
}

#[test]
fn column_vertical_alignment() {
    let mut table = Table::new();
    table
        .set_header(vec!["H1", "H2", "H3"])
        .add_row(vec![Cell::new("1\n2\n3"), Cell::new("A"), Cell::new("X")])
        .add_row(vec![Cell::new("4\n5\n6"), Cell::new("B"), Cell::new("Y")]);

    table
        .column_mut(1)
        .unwrap()
        .set_vertical_alignment(VerticalAlignment::Middle);
    table
        .column_mut(2)
        .unwrap()
        .set_vertical_alignment(VerticalAlignment::Bottom);

    let expected = "
+----+----+----+
| H1 | H2 | H3 |
+==============+
| 1  |    |    |
| 2  | A  |    |
| 3  |    | X  |
|----+----+----|
| 4  |    |    |
| 5  | B  |    |
| 6  |    | Y  |
+----+----+----+";
    assert_eq!(expected, "\n".to_string() + &table.to_string());
}
