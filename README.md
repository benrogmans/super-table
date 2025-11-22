# Super-table

[![GitHub Actions Workflow](https://github.com/benrogmans/super-table/actions/workflows/quality.yml/badge.svg)](https://github.com/benrogmans/super-table/actions/workflows/quality.yml)
[![docs](https://docs.rs/super-table/badge.svg)](https://docs.rs/super-table/)
[![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/benrogmans/super-table/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/super-table.svg)](https://crates.io/crates/super-table)
[![codecov](https://codecov.io/gh/benrogmans/super-table/branch/main/graph/badge.svg)](https://codecov.io/gh/benrogmans/super-table)

Super-table is designed as a library for building beautiful terminal tables, while being easy to use. It includes some features that are not provided by other CLI tables, such as cell spanning across columns and rows.

## Table of Contents

- [Features](#features)
- [Examples](#examples)
- [Feature Flags](#feature-flags)
- [Contributing](#contributing)

## State of the Project

Super-table is actively maintained and open to new features and improvements.

## Features

- Dynamic arrangement of content depending on a given width.
- ANSI content styling for terminals (Colors, Bold, Blinking, etc.).
- Styling Presets and preset modifiers to get you started.
- Pretty much every part of the table is customizable (borders, lines, padding, alignment).
- Constraints on columns that allow some additional control over how to arrange content.
- Cell spanning (colspan and rowspan) for complex table layouts.
- Cross platform (Linux, macOS, Windows).
- It's fast enough.
  - Benchmarks show that a pretty big table with complex constraints is build in `470μs` or `~0.5ms`.
  - The table seen at the top of the readme takes `~30μs`.
  - These numbers are from a overclocked `i7-8700K` with a max single-core performance of 4.9GHz.
  - To run the benchmarks yourselves, install criterion via `cargo install cargo-criterion` and run `cargo criterion` afterwards.

Super-table is written for the current `stable` Rust version.
Older Rust versions may work but aren't officially supported.

## Examples

```rust
use super_table::Table;

fn main() {
    let mut table = Table::new();
    table
        .set_header(vec!["Header1", "Header2", "Header3"])
        .add_row(vec![
            "This is a text",
            "This is another text",
            "This is the third text",
        ])
        .add_row(vec![
            "This is another text",
            "Now\nadd some\nmulti line stuff",
            "This is awesome",
        ]);

    println!("{table}");
}
```

Create a very basic table.\
This table will become as wide as your content. Nothing fancy happening here.

```text,ignore
+----------------------+----------------------+------------------------+
| Header1              | Header2              | Header3                |
+======================================================================+
| This is a text       | This is another text | This is the third text |
|----------------------+----------------------+------------------------|
| This is another text | Now                  | This is awesome        |
|                      | add some             |                        |
|                      | multi line stuff     |                        |
+----------------------+----------------------+------------------------+
```

### More Features

```rust
use super_table::modifiers::UTF8_ROUND_CORNERS;
use super_table::presets::UTF8_FULL;
use super_table::*;

fn main() {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(40)
        .set_header(vec!["Header1", "Header2", "Header3"])
        .add_row(vec![
            Cell::new("Center aligned").set_alignment(CellAlignment::Center),
            Cell::new("This is another text"),
            Cell::new("This is the third text"),
        ])
        .add_row(vec![
            "This is another text",
            "Now\nadd some\nmulti line stuff",
            "This is awesome",
        ]);

    // Set the default alignment for the third column to right
    let column = table.column_mut(2).expect("Our table has three columns");
    column.set_cell_alignment(CellAlignment::Right);

    println!("{table}");
}
```

Create a table with UTF8 styling, and apply a modifier that gives the table round corners.\
Additionally, the content will dynamically wrap to maintain a given table width.\
If the table width isn't explicitly set and the program runs in a terminal, the terminal size will be used.

On top of this, we set the default alignment for the right column to `Right` and the alignment of the left top cell to `Center`.

```text,ignore
╭────────────┬────────────┬────────────╮
│ Header1    ┆ Header2    ┆    Header3 │
╞════════════╪════════════╪════════════╡
│  This is a ┆ This is    ┆    This is │
│    text    ┆ another    ┆  the third │
│            ┆ text       ┆       text │
├╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┤
│ This is    ┆ Now        ┆    This is │
│ another    ┆ add some   ┆    awesome │
│ text       ┆ multi line ┆            │
│            ┆ stuff      ┆            │
╰────────────┴────────────┴────────────╯
```

### Styling

```rust
use super_table::presets::UTF8_FULL;
use super_table::*;

fn main() {
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
        ]);

    println!("{table}");
}
```

This code generates the table that can be seen at the top of this document.

### Cell Spanning

Super-table supports cell spanning, allowing cells to span multiple columns (colspan) and/or rows (rowspan). This enables more complex table layouts.

#### Basic Colspan Example

```rust
use super_table::{Cell, Table};

fn main() {
    let mut table = Table::new();
    table
        .set_header(vec![
            Cell::new("Header1").set_colspan(2),
            Cell::new("Header3"),
        ])
        .add_row(vec![
            Cell::new("Spans 2 cols").set_colspan(2),
            Cell::new("Normal cell"),
        ]);

    println!("{table}");
}
```

```text,ignore
+----------+----------+-------------+
| Header1             | Header3     |
+===================================+
| Spans 2 cols        | Normal cell |
+----------+----------+-------------+
```

#### Basic Rowspan Example

```rust
use super_table::{Cell, Table};

fn main() {
    let mut table = Table::new();
    table
        .set_header(vec!["Header1", "Header2", "Header3"])
        .add_row(vec![
            Cell::new("Spans 2 rows").set_rowspan(2),
            Cell::new("Cell 2"),
            Cell::new("Cell 3"),
        ])
        .add_row(vec![
            // First position is occupied by rowspan above, so only add 2 cells
            Cell::new("Cell 2 (row 2)"),
            Cell::new("Cell 3 (row 2)"),
        ]);

    println!("{table}");
}
```

```text,ignore
+--------------+----------------+----------------+
| Header1      | Header2        | Header3        |
+================================================+
| Spans 2 rows | Cell 2         | Cell 3         |
|              +----------------+----------------|
|              | Cell 2 (row 2) | Cell 3 (row 2) |
+--------------+----------------+----------------+
```

#### Combined Colspan and Rowspan Example

```rust
use super_table::{Cell, Table};

fn main() {
    let mut table = Table::new();
    table
        .set_header(vec!["Header1", "Header2", "Header3", "Header4"])
        .add_row(vec![
            Cell::new("Spans 2x2").set_colspan(2).set_rowspan(2),
            Cell::new("Cell 3"),
            Cell::new("Cell 4"),
        ])
        .add_row(vec![
            // First 2 positions are occupied by rowspan above
            Cell::new("Cell 3 (row 2)"),
            Cell::new("Cell 4 (row 2)"),
        ]);

    println!("{table}");
}
```

```text,ignore
+---------+---------+----------------+----------------+
| Header1 | Header2 | Header3        | Header4        |
+=====================================================+
| Spans 2x2         | Cell 3         | Cell 4         |
|                   +----------------+----------------|
|                   | Cell 3 (row 2) | Cell 4 (row 2) |
+---------+---------+----------------+----------------+
```

**Notes:**
- When using `colspan`, add fewer cells to the row than the number of columns. The spanned cell counts as multiple columns.
- When using `rowspan`, subsequent rows should have fewer cells, as the rowspan cell occupies space in those rows.
- You can combine `colspan` and `rowspan` to create cells that span both multiple rows and columns.
- Cell spanning works with all table features including styling, alignment, and dynamic width arrangement.

### Code Examples

A few examples can be found in the `example` folder.
To test an example, run `cargo run --example $name`. E.g.:

```bash
cargo run --example readme_table
```

If you're looking for more information, take a look at the [tests folder](https://github.com/benrogmans/super-table/tree/main/tests).
There are tests for almost every feature including a visual view for each resulting table.

## Feature Flags

### `tty` (enabled)

This flag enables support for terminals. In detail this means:

- Automatic detection whether we're in a terminal environment.
  Only used when no explicit `Table::set_width` is provided.
- Support for ANSI Escape Code styling for terminals.

### `custom_styling` (disabled)

This flag enables support for custom styling of text inside of cells.

- Text formatting still works, even if you roll your own ANSI escape sequences.
- Rainbow text
- Makes super-table 30-50% slower

### `reexport_crossterm` (disabled)

With this flag, super-table re-exposes crossterm's [`Attribute`](https://docs.rs/crossterm/latest/crossterm/style/enum.Attribute.html) and [`Color`](https://docs.rs/crossterm/latest/crossterm/style/enum.Color.html) enum.
By default, a mirrored type is exposed, which internally maps to the crossterm type.

This feature is very convenient if you use both super-table and crossterm in your code and want to use crossterm's types for everything interchangeably.

**BUT** if you enable this feature, you opt-in for breaking changes on minor/patch versions.
Meaning, you have to update crossterm whenever you update super-table and you **cannot** update crossterm until super-table released a new version with that crossterm version.

## Contributing

Super-table's main focus is on being reliable and feature-rich.
Core features include:

- Normal tables (columns, rows, one cell per column/row).
- Cell spanning (colspan and rowspan) for complex layouts.
- Dynamic arrangement of content to a given width.
- Some kind of manual intervention in the arrangement process.

If you come up with an idea or an improvement, feel free to create an issue!

## Attribution

Super-table is a fork of [comfy-table](https://github.com/nukesor/comfy-table) by Arne Beer. Super-table maintains the same core functionality while allowing for independent development and feature additions, most notably cell spanning over rows and columns.
