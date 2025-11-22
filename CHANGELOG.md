# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
