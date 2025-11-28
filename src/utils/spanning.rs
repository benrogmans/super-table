use std::collections::HashMap;

/// Information about an active rowspan.
#[derive(Debug, Clone)]
struct RowSpanInfo {
    /// Starting row index of the span (also stored in HashMap key for lookup)
    start_row: usize,
    /// Original rowspan value (how many rows the span covers in total)
    original_rowspan: u16,
    /// Number of rows remaining (decremented as we process rows)
    remaining_rows: u16,
    /// Number of columns this span covers
    colspan: u16,
    /// Cached formatted content lines for this rowspan cell (None for border drawing)
    formatted_content: Option<Vec<String>>,
}

/// Tracks active row spans across rows during table rendering.
#[derive(Debug, Clone, Default)]
pub(crate) struct SpanTracker {
    /// Maps (start_row, start_col) -> RowSpanInfo
    active_spans: HashMap<(usize, usize), RowSpanInfo>,
    /// Spans that have ended (for bottom border drawing)
    /// Maps (start_row, start_col) -> (end_row, colspan)
    ended_spans: HashMap<(usize, usize), (usize, u16)>,
}

impl SpanTracker {
    /// Create a new empty SpanTracker.
    pub(crate) fn new() -> Self {
        Self {
            active_spans: HashMap::new(),
            ended_spans: HashMap::new(),
        }
    }

    /// Check if a position is occupied by a rowspan from a previous row.
    ///
    /// Returns `Some((rowspan_remaining, colspan))` if the position is occupied,
    /// `None` otherwise.
    pub(crate) fn is_occupied(&self, row_index: usize, col_index: usize) -> Option<(u16, u16)> {
        for ((start_row, start_col), info) in &self.active_spans {
            if *start_row < row_index {
                // Check if this position falls within the colspan range
                if *start_col <= col_index && col_index < *start_col + info.colspan as usize {
                    return Some((info.remaining_rows, info.colspan));
                }
            }
        }
        None
    }

    /// Register a new rowspan cell with its formatted content.
    ///
    /// This should be called when processing a cell that has rowspan > 1.
    /// remaining_rows is set to rowspan - 1, meaning it will appear in rowspan - 1 more rows.
    pub(crate) fn register_rowspan(
        &mut self,
        row_index: usize,
        col_index: usize,
        rowspan: u16,
        colspan: u16,
        formatted_content: Option<Vec<String>>,
    ) {
        if rowspan > 1 {
            self.active_spans.insert(
                (row_index, col_index),
                RowSpanInfo {
                    start_row: row_index,
                    original_rowspan: rowspan,
                    remaining_rows: rowspan - 1, // Will appear in rowspan - 1 more rows
                    colspan,
                    formatted_content,
                },
            );
        }
    }

    /// Get the cached formatted content for a rowspan cell.
    ///
    /// Returns the formatted content lines if the position is occupied by a rowspan.
    pub(crate) fn get_rowspan_content(
        &self,
        row_index: usize,
        col_index: usize,
    ) -> Option<&Vec<String>> {
        for ((start_row, start_col), info) in &self.active_spans {
            if *start_row < row_index {
                // Check if this position falls within the colspan range
                if *start_col <= col_index && col_index < *start_col + info.colspan as usize {
                    return info.formatted_content.as_ref();
                }
            }
        }
        None
    }

    /// Decrement rowspan counters and remove expired spans.
    ///
    /// This should be called after processing each row.
    /// A rowspan is removed only after it has been displayed in all its spanned rows.
    /// When remaining_rows reaches 0, it means the span was just displayed in its last row,
    /// so we remove it after that row is processed.
    pub(crate) fn advance_row(&mut self, current_row: usize) {
        // First, track and remove spans that have expired (remaining_rows == 0 means it was just displayed in its last row)
        let expired: Vec<_> = self
            .active_spans
            .iter()
            .filter(|(_, info)| info.remaining_rows == 0)
            .map(|((start_row, start_col), info)| {
                let end_row = info.start_row + info.original_rowspan as usize - 1;
                ((*start_row, *start_col), (end_row, info.colspan))
            })
            .collect();

        for ((start_row, start_col), (end_row, colspan)) in expired {
            self.ended_spans
                .insert((start_row, start_col), (end_row, colspan));
            self.active_spans.remove(&(start_row, start_col));
        }

        // Then decrement remaining_rows for all active spans that have been displayed
        // We decrement after the row has been processed, so remaining_rows represents
        // how many more rows the span should appear in
        for info in self.active_spans.values_mut() {
            if info.start_row < current_row && info.remaining_rows > 0 {
                info.remaining_rows -= 1;
            }
        }
    }

    /// Check if a column position is part of any active rowspan.
    pub(crate) fn is_col_occupied_by_rowspan(&self, row_index: usize, col_index: usize) -> bool {
        self.is_occupied(row_index, col_index).is_some()
    }

    /// Get the starting position of a rowspan that occupies the given position.
    ///
    /// Returns `Some((start_row, start_col, colspan))` if the position is occupied,
    /// `None` otherwise.
    pub(crate) fn get_rowspan_start(
        &self,
        row_index: usize,
        col_index: usize,
    ) -> Option<(usize, usize, u16)> {
        for ((start_row, start_col), info) in &self.active_spans {
            if *start_row < row_index {
                // Check if this position falls within the colspan range
                if *start_col <= col_index && col_index < *start_col + info.colspan as usize {
                    return Some((*start_row, *start_col, info.colspan));
                }
            }
        }
        None
    }

    /// Get the starting position of a rowspan that occupies the given position at the given row.
    /// This includes rowspans that started at the current row (for border drawing).
    /// Only returns spans that CONTINUE past the current row (remaining_rows > 0).
    ///
    /// Returns `Some((start_row, start_col, colspan))` if the position is occupied by a
    /// continuing rowspan, `None` otherwise.
    pub(crate) fn get_rowspan_start_at_row(
        &self,
        row_index: usize,
        col_index: usize,
    ) -> Option<(usize, usize, u16)> {
        for ((start_row, start_col), info) in &self.active_spans {
            // Check if rowspan is active at this row (started at or before this row, and still has remaining rows)
            if *start_row <= row_index && info.remaining_rows > 0 {
                // Check if this position falls within the colspan range
                if *start_col <= col_index && col_index < *start_col + info.colspan as usize {
                    return Some((*start_row, *start_col, info.colspan));
                }
            }
        }
        None
    }

    /// Get the starting position of a rowspan that includes the given row and column.
    /// This includes rowspans that END at this row (remaining_rows = 0) for detecting
    /// merge intersections between consecutive rowspans.
    ///
    /// Returns `Some((start_row, start_col, colspan))` if the position is part of any rowspan
    /// that includes this row, `None` otherwise.
    pub(crate) fn get_rowspan_including_row(
        &self,
        row_index: usize,
        col_index: usize,
    ) -> Option<(usize, usize, u16)> {
        for ((start_row, start_col), info) in &self.active_spans {
            // Check if rowspan includes this row (based on original rowspan value)
            let end_row = info.start_row + info.original_rowspan as usize - 1;
            if *start_row <= row_index && end_row >= row_index {
                // Check if this position falls within the colspan range
                if *start_col <= col_index && col_index < *start_col + info.colspan as usize {
                    return Some((*start_row, *start_col, info.colspan));
                }
            }
        }
        None
    }

    /// Get rowspan info for a position at the last row of the table.
    /// This checks both active spans and spans that have ended (for bottom border drawing).
    ///
    /// Returns `Some((start_row, start_col, colspan))` if the position is part of a rowspan
    /// that includes the specified row, `None` otherwise.
    pub(crate) fn get_rowspan_at_last_row(
        &self,
        row_index: usize,
        col_index: usize,
    ) -> Option<(usize, usize, u16)> {
        // Check active spans first (reuse existing logic)
        if let Some(result) = self.get_rowspan_including_row(row_index, col_index) {
            return Some(result);
        }

        // Check ended spans (already removed from active_spans)
        for ((start_row, start_col), (end_row, colspan)) in &self.ended_spans {
            if *start_row <= row_index
                && *end_row >= row_index
                && *start_col <= col_index
                && col_index < *start_col + *colspan as usize
            {
                return Some((*start_row, *start_col, *colspan));
            }
        }

        None
    }
}
