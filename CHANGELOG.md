# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2025-11-28

### Added
- Vertical alignment support for cell content
  - `VerticalAlignment` enum with `Top`, `Middle`, and `Bottom` variants
  - `Cell::set_vertical_alignment(alignment: VerticalAlignment)` - Set vertical alignment for a specific cell
  - `Column::set_vertical_alignment(alignment: VerticalAlignment)` - Set vertical alignment for all cells in a column
  - Vertical alignment works with multi-line content and rowspans
  - Default vertical alignment is `Top` (content aligns to the top of the cell)
  - Cell-level vertical alignment overrides column-level settings

### Improved
- Border rendering for rowspan and colspan cells
  - Added `MiddleHeaderMergeIntersection` and `BottomBorderColspanIntersections` table components for proper merge rendering at boundaries
  - Improved colspan merging in top and bottom borders
  - Improved rowspan continuation rendering in horizontal separators
  - Refactored border drawing logic with `ColumnBorderInfo` for cleaner and more maintainable code
  - Added `BorderStyles` struct and `IntersectionType` enum for better code organization

## [1.0.0] - 2025-11-22

## Added
- Cell spanning support: cells can now span multiple columns (`colspan`) and/or rows (`rowspan`)
  - `Cell::set_colspan(cols: u16)` - Set the number of columns a cell spans
  - `Cell::set_rowspan(rows: u16)` - Set the number of rows a cell spans
  - `Cell::colspan() -> u16` - Get the number of columns a cell spans
  - `Cell::rowspan() -> u16` - Get the number of rows a cell spans
  - `Cell::span_columns(cols: u16)` - Convenience alias for `set_colspan`
  - `Cell::span_rows(rows: u16)` - Convenience alias for `set_rowspan`
  - Cell spanning works with all table features including styling, alignment, and dynamic width arrangement
  - Hidden columns are automatically excluded from colspan calculations

### Forked

- Initial fork from [comfy-table](https://github.com/nukesor/comfy-table) version 7.2.1
- Maintains existing features from the original project
- Renamed package to `super-table`
