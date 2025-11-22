use criterion::{Criterion, criterion_group, criterion_main};

use super_table::ColumnConstraint::*;
use super_table::Width::*;
use super_table::presets::UTF8_FULL;
use super_table::*;

/// Build the readme table
#[cfg(feature = "tty")]
fn build_readme_table() {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(80)
        .set_header(vec![
            Cell::new("Header1").add_attribute(Attribute::Bold),
            Cell::new("Header2").fg(Color::Green),
            Cell::new("Header3"),
        ])
        .add_row(vec![
            Cell::new("This is a bold text").add_attribute(Attribute::Bold),
            Cell::new("This is a green text").fg(Color::Green),
            Cell::new("This one has black background").bg(Color::Black),
        ])
        .add_row(vec![
            Cell::new("Blinky boi").add_attribute(Attribute::SlowBlink),
            Cell::new("This table's content is dynamically arranged. The table is exactly 80 characters wide.\nHere comes a reallylongwordthatshoulddynamicallywrap"),
            Cell::new("COMBINE ALL THE THINGS")
            .fg(Color::Green)
            .bg(Color::Black)
            .add_attributes(vec![
                Attribute::Bold,
                Attribute::SlowBlink,
            ])
        ])
        .add_row(vec![
            Cell::new("Spans 2 cols").set_colspan(2),
            Cell::new("Normal cell"),
        ])
        .add_row(vec![
            Cell::new("Spans 2 rows").set_rowspan(2),
            Cell::new("Cell 2"),
            Cell::new("Cell 3"),
        ])
        .add_row(vec![
            Cell::new("Cell 2 (row 2)"),
            Cell::new("Cell 3 (row 2)"),
        ]);

    // Build the table.
    let _ = table.lines();
}

#[cfg(not(feature = "tty"))]
fn build_readme_table() {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(80)
        .set_header(vec![
            Cell::new("Header1"),
            Cell::new("Header2"),
            Cell::new("Header3"),
        ])
        .add_row(vec![
            Cell::new("This is a bold text"),
            Cell::new("This is a green text"),
            Cell::new("This one has black background"),
        ])
        .add_row(vec![
            Cell::new("Blinky boi"),
            Cell::new("This table's content is dynamically arranged. The table is exactly 80 characters wide.\nHere comes a reallylongwordthatshoulddynamicallywrap"),
            Cell::new("COMBINE ALL THE THINGS"),
        ])
        .add_row(vec![
            Cell::new("Spans 2 cols").set_colspan(2),
            Cell::new("Normal cell"),
        ])
        .add_row(vec![
            Cell::new("Spans 2 rows").set_rowspan(2),
            Cell::new("Cell 2"),
            Cell::new("Cell 3"),
        ])
        .add_row(vec![
            Cell::new("Cell 2 (row 2)"),
            Cell::new("Cell 3 (row 2)"),
        ]);

    // Build the table.
    let _ = table.lines();
}

/// Create a dynamic 10x10 Table with width 400 and unevenly distributed content.
/// On top of that, most of the columns have some kind of constraint.
fn build_big_table() {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_width(400)
        .set_header(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    // Create a 10x10 grid with some cell spans
    for row_index in 0..10 {
        let mut row = Vec::new();
        if row_index == 2 {
            // Add a colspan in row 2
            row.push(Cell::new("Spans 2 cols").set_colspan(2));
            for column in 2..10 {
                row.push(Cell::new("SomeWord ".repeat((column + row_index * 2) % 10)));
            }
        } else if row_index == 5 {
            // Add a rowspan in row 5
            row.push(Cell::new("Spans 2 rows").set_rowspan(2));
            for column in 1..10 {
                row.push(Cell::new("SomeWord ".repeat((column + row_index * 2) % 10)));
            }
        } else if row_index == 6 {
            // Skip first column due to rowspan above
            for column in 1..10 {
                row.push(Cell::new("SomeWord ".repeat((column + row_index * 2) % 10)));
            }
        } else {
            for column in 0..10 {
                row.push(Cell::new("SomeWord ".repeat((column + row_index * 2) % 10)));
            }
        }
        table.add_row(row);
    }

    table.set_constraints(vec![
        UpperBoundary(Fixed(20)),
        LowerBoundary(Fixed(40)),
        Absolute(Fixed(5)),
        Absolute(Percentage(3)),
        Absolute(Percentage(3)),
        Boundaries {
            lower: Fixed(30),
            upper: Percentage(10),
        },
    ]);

    // Build the table.
    let _ = table.lines();
}

pub fn build_tables(crit: &mut Criterion) {
    crit.bench_function("Readme table", |b| b.iter(build_readme_table));

    crit.bench_function("Big table", |b| b.iter(build_big_table));
}

criterion_group!(benches, build_tables);
criterion_main!(benches);
