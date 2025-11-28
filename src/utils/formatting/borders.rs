use crate::style::TableComponent;
use crate::table::Table;
use crate::utils::ColumnDisplayInfo;
use crate::utils::spanning::SpanTracker;

/// Information about a column's state at a horizontal border position.
/// Pre-computed to simplify border drawing logic.
#[derive(Debug, Clone, Default)]
struct ColumnBorderInfo {
    /// Column is hidden
    is_hidden: bool,
    /// Column width
    width: usize,
    /// Column is part of a rowspan that continues past this row (draw spaces, no border)
    is_rowspan_continuing: bool,
    /// Start column of the continuing rowspan (if any)
    rowspan_start_col: Option<usize>,
    /// Colspan size of the continuing rowspan
    rowspan_colspan: u16,
    /// Column is part of a rowspan that ends at this row (draw merged border)
    is_rowspan_ending: bool,
    /// Start column of the ending rowspan (if any)
    ending_rowspan_start_col: Option<usize>,
    /// Colspan size of the ending rowspan
    ending_rowspan_colspan: u16,
    /// Column is a colspan continuation in the current row (not the first column of a cell)
    is_colspan_continuation: bool,
    /// Next row has a colspan at this position (for merge intersection)
    next_row_has_colspan: bool,
}

/// Pre-compute border info for all columns at a given row separator.
fn compute_column_border_info(
    display_info: &[ColumnDisplayInfo],
    row_index: usize,
    span_tracker: &SpanTracker,
    row_line: &[String],
    next_row_line: Option<&[String]>,
) -> Vec<ColumnBorderInfo> {
    let mut infos: Vec<ColumnBorderInfo> = Vec::with_capacity(display_info.len());
    let mut visible_col_index = 0;

    for (col_index, info) in display_info.iter().enumerate() {
        let mut col_info = ColumnBorderInfo {
            is_hidden: info.is_hidden,
            width: info.width() as usize,
            ..Default::default()
        };

        if info.is_hidden {
            infos.push(col_info);
            continue;
        }

        // Check for continuing rowspan (spans past this row)
        if let Some((_start_row, start_col, colspan)) =
            span_tracker.get_rowspan_start_at_row(row_index, col_index)
        {
            col_info.is_rowspan_continuing = true;
            col_info.rowspan_start_col = Some(start_col);
            col_info.rowspan_colspan = colspan;
        }
        // Check for ending rowspan (ends at this row, not continuing)
        else if let Some((_start_row, start_col, colspan)) =
            span_tracker.get_rowspan_including_row(row_index, col_index)
        {
            col_info.is_rowspan_ending = true;
            col_info.ending_rowspan_start_col = Some(start_col);
            col_info.ending_rowspan_colspan = colspan;
        }

        // Check if this column is a colspan continuation in the current row
        if visible_col_index < row_line.len() {
            col_info.is_colspan_continuation = row_line[visible_col_index].is_empty();
        }

        // Check if next row has colspan at this position
        col_info.next_row_has_colspan = next_row_line
            .map(|next| visible_col_index < next.len() && next[visible_col_index].is_empty())
            .unwrap_or(false);

        infos.push(col_info);
        visible_col_index += 1;
    }

    infos
}

/// Determine which intersection character to use based on context.
#[derive(Debug, Clone, Copy, PartialEq)]
enum IntersectionType {
    /// Normal intersection (┼ or +)
    Normal,
    /// Merge intersection for colspan below (╧ or ┴)
    Merge,
    /// Left border intersection after rowspan (├)
    LeftBorderAfterRowspan,
}

fn select_intersection_type(
    header: bool,
    previous_was_rowspan: bool,
    next_row_has_colspan: bool,
) -> IntersectionType {
    if !header && previous_was_rowspan {
        IntersectionType::LeftBorderAfterRowspan
    } else if next_row_has_colspan {
        IntersectionType::Merge
    } else {
        IntersectionType::Normal
    }
}

pub(crate) fn draw_borders(
    table: &Table,
    rows: &[Vec<Vec<String>>],
    display_info: &[ColumnDisplayInfo],
) -> Vec<String> {
    // We know how many lines there should be. Initialize the vector with the rough correct amount.
    // We might over allocate a bit, but that's better than under allocating.
    let mut lines = if let Some(capacity) = rows.first().map(|lines| lines.len()) {
        // Lines * 2 -> Lines + delimiters
        // + 5 -> header delimiters + header + bottom/top borders
        Vec::with_capacity(capacity * 2 + 5)
    } else {
        Vec::new()
    };

    // Build span information for border drawing
    let mut span_tracker = SpanTracker::new();
    let header_rows = if table.header.is_some() { 1 } else { 0 };

    if should_draw_top_border(table) {
        lines.push(draw_top_border(table, display_info));
    }

    draw_rows(
        &mut lines,
        rows,
        table,
        display_info,
        &mut span_tracker,
        header_rows,
    );

    if should_draw_bottom_border(table) {
        // Get the last row's first line to detect colspan for bottom border
        let last_row_line = rows
            .last()
            .and_then(|row| row.first().map(|line| line.as_slice()));
        // Calculate the last row index for rowspan detection
        let last_row_index = if rows.is_empty() {
            0
        } else {
            rows.len() - 1
        };
        lines.push(draw_bottom_border(
            table,
            display_info,
            last_row_line,
            &span_tracker,
            last_row_index,
        ));
    }

    lines
}

/// Build a map of which columns are colspan continuations (not the first column of a cell).
/// Returns (continuation_map, all_cells_have_colspan).
fn build_colspan_continuation_map(
    row_cells: Option<&crate::row::Row>,
    num_columns: usize,
) -> (Vec<bool>, bool) {
    let mut continuation: Vec<bool> = vec![false; num_columns];
    let mut all_have_colspan = true;

    if let Some(row) = row_cells {
        let mut col_index = 0;
        for cell in &row.cells {
            let colspan = cell.colspan() as usize;
            if colspan == 1 {
                all_have_colspan = false;
            }
            for i in 1..colspan {
                if col_index + i < continuation.len() {
                    continuation[col_index + i] = true;
                }
            }
            col_index += colspan;
        }
    } else {
        all_have_colspan = false;
    }

    (continuation, all_have_colspan)
}

fn draw_top_border(table: &Table, display_info: &[ColumnDisplayInfo]) -> String {
    let left_corner = table.style_or_default(TableComponent::TopLeftCorner);
    let top_border = table.style_or_default(TableComponent::TopBorder);
    let intersection = table.style_or_default(TableComponent::TopBorderIntersections);
    let right_corner = table.style_or_default(TableComponent::TopRightCorner);

    let (header_colspan_continuation, all_header_cells_have_colspan) =
        build_colspan_continuation_map(table.header.as_ref(), display_info.len());

    // Merge header colspans in top border unless:
    // - All header cells have colspan > 1 (suggests header doesn't define column structure)
    // - Dynamic content arrangement (column widths are calculated dynamically)
    let is_dynamic = matches!(
        table.content_arrangement(),
        crate::ContentArrangement::Dynamic | crate::ContentArrangement::DynamicFullWidth
    );
    let should_merge_header_colspan = !all_header_cells_have_colspan && !is_dynamic;

    let mut line = String::new();
    // We only need the top left corner, if we need to draw a left border
    if should_draw_left_border(table) {
        line += &left_corner;
    }

    // Build the top border line. Merge where header has colspan (unless all cells have colspan).
    let mut first = true;
    for (col_index, info) in display_info.iter().enumerate() {
        if !info.is_hidden {
            if !first {
                // Check if header has colspan at this position
                let header_has_colspan = col_index < header_colspan_continuation.len()
                    && header_colspan_continuation[col_index];

                if should_merge_header_colspan && header_has_colspan {
                    // Use top_border to continue the line (merge)
                    line += &top_border;
                } else {
                    line += &intersection;
                }
            }
            line += &top_border.repeat(info.width().into());
            first = false;
        }
    }

    // We only need the top right corner, if we need to draw a right border
    if should_draw_right_border(table) {
        line += &right_corner;
    }

    line
}

fn draw_rows(
    lines: &mut Vec<String>,
    rows: &[Vec<Vec<String>>],
    table: &Table,
    display_info: &[ColumnDisplayInfo],
    span_tracker: &mut SpanTracker,
    header_rows: usize,
) {
    // Iterate over all rows
    let mut row_iter = rows.iter().enumerate().peekable();
    while let Some((row_index, row)) = row_iter.next() {
        let actual_row_index = if row_index < header_rows {
            row_index
        } else {
            row_index - header_rows
        };

        // Concatenate the line parts and insert the vertical borders if needed
        for line_parts in row.iter() {
            lines.push(embed_line(
                line_parts,
                table,
                actual_row_index,
                span_tracker,
            ));
        }

        // Draw the horizontal header line if desired, otherwise continue to the next iteration
        if row_index == 0 && table.header.is_some() {
            if should_draw_header(table) {
                // Header separator should match the header content width (widest line)
                // Draw all physical columns separately (like top border)
                // Get next row's first line to detect colspan transitions
                let next_row_line = row_iter.peek().and_then(|(_, next_row)| {
                    next_row.first().map(|line| line.as_slice())
                });
                lines.push(draw_horizontal_lines(
                    table,
                    display_info,
                    true,
                    0,
                    span_tracker,
                    row.first().map(|line| line.as_slice()).unwrap_or(&[]),
                    next_row_line,
                ));
            }
            // Register rowspans from header for border drawing (we only need position info, not content)
            if let Some(header) = &table.header {
                let mut col_index = 0;
                for cell in &header.cells {
                    if cell.rowspan() > 1 {
                        span_tracker.register_rowspan(
                            0,
                            col_index,
                            cell.rowspan(),
                            cell.colspan(),
                            None,
                        );
                    }
                    col_index += cell.colspan() as usize;
                }
            }
            span_tracker.advance_row(1);
            continue;
        }

        // Register rowspans from data rows for border drawing
        if actual_row_index < table.rows.len() {
            let data_row = &table.rows[actual_row_index];
            let mut col_index = 0;
            for cell in &data_row.cells {
                // Skip positions occupied by rowspan
                while col_index < display_info.len()
                    && span_tracker
                        .is_col_occupied_by_rowspan(actual_row_index + header_rows, col_index)
                {
                    col_index += 1;
                }
                if col_index >= display_info.len() {
                    break;
                }
                if cell.rowspan() > 1 {
                    span_tracker.register_rowspan(
                        actual_row_index + header_rows,
                        col_index,
                        cell.rowspan(),
                        cell.colspan(),
                        None,
                    );
                }
                col_index += cell.colspan() as usize;
            }
        }

        // Draw a horizontal line, if we desired and if we aren't in the last row of the table.
        // When drawing the border after a row, we need to check for rowspans that continue into the next row.
        // So we check at the current row_index (the row we just processed).
        if let Some(next_row) = row_iter.peek() {
            if should_draw_horizontal_lines(table) {
                // Draw all physical columns separately (like top border), not based on row structure
                let border_line = row.first().map(|line| line.as_slice()).unwrap_or(&[]);
                // Get next row's first line to detect colspan transitions
                let next_row_line = next_row.1.first().map(|line| line.as_slice());
                // Check for rowspans at the current row_index (row we just processed)
                // Rowspans that started at this row or earlier and still have remaining_rows should skip borders
                lines.push(draw_horizontal_lines(
                    table,
                    display_info,
                    false,
                    actual_row_index + header_rows,
                    span_tracker,
                    border_line,
                    next_row_line,
                ));
            }
        }

        span_tracker.advance_row(actual_row_index + header_rows + 1);
    }
}

// Takes the parts of a single line, surrounds them with borders and adds vertical lines.
// Skips vertical borders within colspan cells (detected by empty strings).
fn embed_line(
    line_parts: &[String],
    table: &Table,
    _row_index: usize,
    _span_tracker: &SpanTracker,
) -> String {
    let vertical_lines = table.style_or_default(TableComponent::VerticalLines);
    let left_border = table.style_or_default(TableComponent::LeftBorder);
    let right_border = table.style_or_default(TableComponent::RightBorder);

    let mut line = String::new();
    if should_draw_left_border(table) {
        line += &left_border;
    }

    let mut part_iter = line_parts.iter().peekable();
    while let Some(part) = part_iter.next() {
        line += part;
        // Check if the next part exists and is not empty (empty string indicates colspan)
        let next_part = part_iter.peek();
        if let Some(next) = next_part {
            // If next part is empty, it's part of a colspan - skip vertical border
            if next.is_empty() {
                // Skip the border for colspan
            } else if should_draw_vertical_lines(table) {
                line += &vertical_lines;
            }
        } else if should_draw_right_border(table) {
            line += &right_border;
        }
    }

    line
}

/// The horizontal line that separates between rows.
/// Uses pre-computed ColumnBorderInfo for cleaner logic.
fn draw_horizontal_lines(
    table: &Table,
    display_info: &[ColumnDisplayInfo],
    header: bool,
    row_index: usize,
    span_tracker: &SpanTracker,
    row_line: &[String],
    next_row_line: Option<&[String]>,
) -> String {
    // Pre-compute border info for all columns
    let column_infos = compute_column_border_info(
        display_info,
        row_index,
        span_tracker,
        row_line,
        next_row_line,
    );

    // Get style characters based on header vs data row
    let styles = BorderStyles::for_row(table, header);

    let mut line = String::new();
    let mut previous_was_rowspan = false;

    // Draw left border/intersection
    if should_draw_left_border(table) {
        let first_visible = column_infos.iter().find(|c| !c.is_hidden);
        if !header && first_visible.map(|c| c.is_rowspan_continuing).unwrap_or(false) {
            line += &styles.left_border;
            previous_was_rowspan = true;
        } else {
            line += &styles.left_intersection;
        }
    }

    // Process columns using the pre-computed info
    let mut first = true;
    let mut col_idx = 0;

    while col_idx < column_infos.len() {
        let col = &column_infos[col_idx];

        if col.is_hidden {
            col_idx += 1;
            continue;
        }

        // Case 1: Continuing rowspan - draw spaces
        if col.is_rowspan_continuing {
            let start_col = col.rowspan_start_col.expect(
                "rowspan_start_col must be Some when is_rowspan_continuing is true"
            );
            let (spaces, cols_consumed) =
                draw_rowspan_space(display_info, start_col, col.rowspan_colspan);
            line += &spaces;
            col_idx += cols_consumed;
            first = false;
            previous_was_rowspan = true;
            continue;
        }

        // Case 2: Ending rowspan - draw merged border
        if col.is_rowspan_ending {
            let start_col = col.ending_rowspan_start_col.expect(
                "ending_rowspan_start_col must be Some when is_rowspan_ending is true"
            );
            let (border, cols_consumed) = draw_ending_rowspan_border(
                display_info,
                &column_infos,
                col_idx,
                start_col,
                col.ending_rowspan_colspan,
                &styles,
                first,
                previous_was_rowspan,
                header,
            );
            line += &border;
            col_idx += cols_consumed;
            first = false;
            previous_was_rowspan = false;
            continue;
        }

        // Case 3: Colspan continuation - just draw horizontal line (no intersection)
        if col.is_colspan_continuation {
            line += &styles.horizontal.repeat(col.width);
            col_idx += 1;
            continue;
        }

        // Case 4: Normal column or colspan start - draw intersection + border
        // First, count how many following columns are colspan continuations
        let mut colspan_count = 1;
        let mut total_width = col.width;
        while col_idx + colspan_count < column_infos.len() {
            let next = &column_infos[col_idx + colspan_count];
            if next.is_hidden {
                colspan_count += 1;
                continue;
            }
            if next.is_colspan_continuation && !next.is_rowspan_continuing && !next.is_rowspan_ending
            {
                total_width += 1 + next.width; // +1 for merged separator
                colspan_count += 1;
            } else {
                break;
            }
        }

        // Draw intersection before this column (if not first)
        if !first {
            let intersection_type =
                select_intersection_type(header, previous_was_rowspan, col.next_row_has_colspan);
            line += styles.get_intersection(intersection_type);
        }

        // Draw the border
        line += &styles.horizontal.repeat(total_width);
        col_idx += colspan_count;
        first = false;
        previous_was_rowspan = false;
    }

    // Draw right border/intersection
    if should_draw_right_border(table) {
        line += &styles.right_intersection;
    }

    line
}

/// Style characters for border drawing
struct BorderStyles {
    left_intersection: String,
    left_border: String,
    horizontal: String,
    middle_intersection: String,
    merge_intersection: String,
    left_border_intersection: String,
    right_intersection: String,
}

impl BorderStyles {
    fn for_row(table: &Table, header: bool) -> Self {
        if header {
            Self {
                left_intersection: table.style_or_default(TableComponent::LeftHeaderIntersection),
                left_border: table.style_or_default(TableComponent::LeftBorder),
                horizontal: table.style_or_default(TableComponent::HeaderLines),
                middle_intersection: table
                    .style_or_default(TableComponent::MiddleHeaderIntersections),
                merge_intersection: table
                    .style_or_default(TableComponent::MiddleHeaderMergeIntersection),
                left_border_intersection: table
                    .style_or_default(TableComponent::LeftBorderIntersections),
                right_intersection: table
                    .style_or_default(TableComponent::RightHeaderIntersection),
            }
        } else {
            Self {
                left_intersection: table.style_or_default(TableComponent::LeftBorderIntersections),
                left_border: table.style_or_default(TableComponent::LeftBorder),
                horizontal: table.style_or_default(TableComponent::HorizontalLines),
                middle_intersection: table.style_or_default(TableComponent::MiddleIntersections),
                merge_intersection: table
                    .style_or_default(TableComponent::BottomBorderIntersections),
                left_border_intersection: table
                    .style_or_default(TableComponent::LeftBorderIntersections),
                right_intersection: table
                    .style_or_default(TableComponent::RightBorderIntersections),
            }
        }
    }

    fn get_intersection(&self, typ: IntersectionType) -> &str {
        match typ {
            IntersectionType::Normal => &self.middle_intersection,
            IntersectionType::Merge => &self.merge_intersection,
            IntersectionType::LeftBorderAfterRowspan => &self.left_border_intersection,
        }
    }
}

/// Draw spaces for a continuing rowspan area.
/// Returns (spaces_string, number_of_columns_consumed).
fn draw_rowspan_space(
    display_info: &[ColumnDisplayInfo],
    start_col: usize,
    colspan: u16,
) -> (String, usize) {
    let end_col = start_col + colspan as usize;
    let mut width: usize = 0;
    let mut visible_count: usize = 0;

    for info in display_info.iter().take(end_col).skip(start_col) {
        if !info.is_hidden {
            width += info.width() as usize;
            visible_count += 1;
        }
    }
    // Add separators between columns
    width += visible_count.saturating_sub(1);

    (" ".repeat(width), end_col.saturating_sub(start_col))
}

/// Draw border for an ending rowspan (with merged columns).
/// Returns (border_string, number_of_columns_consumed).
#[allow(clippy::too_many_arguments)]
fn draw_ending_rowspan_border(
    display_info: &[ColumnDisplayInfo],
    column_infos: &[ColumnBorderInfo],
    current_idx: usize,
    start_col: usize,
    colspan: u16,
    styles: &BorderStyles,
    first: bool,
    previous_was_rowspan: bool,
    header: bool,
) -> (String, usize) {
    let end_col = start_col + colspan as usize;
    let mut result = String::new();

    // Count visible columns
    let visible_cols: Vec<_> = (start_col..end_col.min(display_info.len()))
        .filter(|&i| !display_info[i].is_hidden)
        .collect();

    if visible_cols.is_empty() {
        return (result, end_col - start_col);
    }

    // Draw intersection at start (if not first column)
    if !first {
        let next_row_has_colspan = column_infos
            .get(current_idx)
            .map(|c| c.next_row_has_colspan)
            .unwrap_or(false);
        let intersection_type =
            select_intersection_type(header, previous_was_rowspan, next_row_has_colspan);
        result += styles.get_intersection(intersection_type);
    }

    // Draw first column border
    result += &styles.horizontal.repeat(display_info[visible_cols[0]].width().into());

    // Draw remaining columns with continuous horizontal lines (merged)
    for &col in &visible_cols[1..] {
        result += &styles.horizontal; // Use horizontal line instead of intersection
        result += &styles.horizontal.repeat(display_info[col].width().into());
    }

    (result, end_col - start_col)
}

fn draw_bottom_border(
    table: &Table,
    display_info: &[ColumnDisplayInfo],
    last_row_line: Option<&[String]>,
    span_tracker: &SpanTracker,
    last_row_index: usize,
) -> String {
    let left_corner = table.style_or_default(TableComponent::BottomLeftCorner);
    let bottom_border = table.style_or_default(TableComponent::BottomBorder);
    let intersection = table.style_or_default(TableComponent::BottomBorderIntersections);
    let right_corner = table.style_or_default(TableComponent::BottomRightCorner);
    let merge_intersection = table.style_or_default(TableComponent::BottomBorderColspanIntersections);

    let (header_colspan_continuation, _) =
        build_colspan_continuation_map(table.header.as_ref(), display_info.len());

    let mut line = String::new();
    if should_draw_left_border(table) {
        line += &left_corner;
    }

    // Build the bottom border considering header colspans, last row colspans, and rowspans
    let mut first = true;
    let mut visible_col_index = 0;
    let mut col_index = 0;

    while col_index < display_info.len() {
        let info = &display_info[col_index];

        if info.is_hidden {
            col_index += 1;
            continue;
        }

        // Check if this column is part of a rowspan that spans to this row
        if let Some((_start_row, start_col, rowspan_colspan)) =
            span_tracker.get_rowspan_at_last_row(last_row_index, col_index)
        {
            // This column is part of a rowspan, handle the entire spanned area
            let visible_cols_in_rowspan: usize = (start_col..start_col + rowspan_colspan as usize)
                .filter(|&i| i < display_info.len() && !display_info[i].is_hidden)
                .count();

            // For bottom border: draw continuous border across the rowspan area
            if !first && visible_cols_in_rowspan > 0 {
                // Use merge intersection at the start of rowspan area (columns are merging)
                line += &merge_intersection;
            }

            // Draw the border for the first column in rowspan
            if visible_cols_in_rowspan > 0 {
                line += &bottom_border.repeat(display_info[start_col].width().into());
            }

            // Draw continuous borders for remaining columns in rowspan
            for i in (start_col + 1)..(start_col + rowspan_colspan as usize) {
                if i < display_info.len() && !display_info[i].is_hidden {
                    line += &merge_intersection;
                    line += &bottom_border.repeat(display_info[i].width().into());
                }
            }

            col_index = start_col + rowspan_colspan as usize;
            visible_col_index += visible_cols_in_rowspan;
            first = false;
            continue;
        }

        if !first {
            // Check if this column is a header colspan continuation
            let is_header_colspan = col_index < header_colspan_continuation.len()
                && header_colspan_continuation[col_index];
            
            // Check if this column is a last row colspan continuation
            let is_lastrow_colspan = last_row_line
                .map(|parts| {
                    visible_col_index < parts.len() && parts[visible_col_index].is_empty()
                })
                .unwrap_or(false);

            // Merge if last row has colspan AND (header also has colspan OR table has few rows)
            let few_data_rows = table.rows.len() <= 2;
            let should_merge = is_lastrow_colspan && (is_header_colspan || few_data_rows);

            if should_merge {
                // Use merge intersection (continuous border) for colspan
                line += &merge_intersection;
            } else {
                line += &intersection;
            }
        }

        line += &bottom_border.repeat(info.width().into());
        first = false;
        visible_col_index += 1;
        col_index += 1;
    }

    if should_draw_right_border(table) {
        line += &right_corner;
    }

    line
}

fn should_draw_top_border(table: &Table) -> bool {
    if table.style_exists(TableComponent::TopLeftCorner)
        || table.style_exists(TableComponent::TopBorder)
        || table.style_exists(TableComponent::TopBorderIntersections)
        || table.style_exists(TableComponent::TopRightCorner)
    {
        return true;
    }

    false
}

fn should_draw_bottom_border(table: &Table) -> bool {
    if table.style_exists(TableComponent::BottomLeftCorner)
        || table.style_exists(TableComponent::BottomBorder)
        || table.style_exists(TableComponent::BottomBorderIntersections)
        || table.style_exists(TableComponent::BottomRightCorner)
    {
        return true;
    }

    false
}

pub fn should_draw_left_border(table: &Table) -> bool {
    if table.style_exists(TableComponent::TopLeftCorner)
        || table.style_exists(TableComponent::LeftBorder)
        || table.style_exists(TableComponent::LeftBorderIntersections)
        || table.style_exists(TableComponent::LeftHeaderIntersection)
        || table.style_exists(TableComponent::BottomLeftCorner)
    {
        return true;
    }

    false
}

pub fn should_draw_right_border(table: &Table) -> bool {
    if table.style_exists(TableComponent::TopRightCorner)
        || table.style_exists(TableComponent::RightBorder)
        || table.style_exists(TableComponent::RightBorderIntersections)
        || table.style_exists(TableComponent::RightHeaderIntersection)
        || table.style_exists(TableComponent::BottomRightCorner)
    {
        return true;
    }

    false
}

fn should_draw_horizontal_lines(table: &Table) -> bool {
    if table.style_exists(TableComponent::LeftBorderIntersections)
        || table.style_exists(TableComponent::HorizontalLines)
        || table.style_exists(TableComponent::MiddleIntersections)
        || table.style_exists(TableComponent::RightBorderIntersections)
    {
        return true;
    }

    false
}

pub fn should_draw_vertical_lines(table: &Table) -> bool {
    if table.style_exists(TableComponent::TopBorderIntersections)
        || table.style_exists(TableComponent::MiddleHeaderIntersections)
        || table.style_exists(TableComponent::VerticalLines)
        || table.style_exists(TableComponent::MiddleIntersections)
        || table.style_exists(TableComponent::BottomBorderIntersections)
    {
        return true;
    }

    false
}

fn should_draw_header(table: &Table) -> bool {
    if table.style_exists(TableComponent::LeftHeaderIntersection)
        || table.style_exists(TableComponent::HeaderLines)
        || table.style_exists(TableComponent::MiddleHeaderIntersections)
        || table.style_exists(TableComponent::RightHeaderIntersection)
    {
        return true;
    }

    false
}
