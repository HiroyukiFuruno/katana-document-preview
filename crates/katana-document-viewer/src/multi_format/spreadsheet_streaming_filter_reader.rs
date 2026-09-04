use super::super::spreadsheet_engine::SpreadsheetEngineError;
use super::super::spreadsheet_streaming_cell_types::{Capture, CellAccumulator, empty_cell};
use super::{SpreadsheetCellArtifact, SpreadsheetCoordinate, StreamingSpreadsheetSession};

#[path = "spreadsheet_streaming_filter_reader_parse.rs"]
mod parse;
#[path = "spreadsheet_streaming_filter_reader_source.rs"]
mod source;

type FilterGridVisitor<'a> = dyn FnMut(
        std::ops::Range<usize>,
        Vec<SpreadsheetCellArtifact>,
    ) -> Result<(), SpreadsheetEngineError>
    + 'a;

impl StreamingSpreadsheetSession {
    pub(in crate::multi_format) fn visit_filter_grid(
        &self,
        sheet_index: usize,
        columns: &[usize],
        rows: std::ops::Range<usize>,
        chunk_rows: usize,
        visitor: &mut FilterGridVisitor<'_>,
    ) -> Result<(), SpreadsheetEngineError> {
        #[cfg(test)]
        self.filter_grid_scans
            .set(self.filter_grid_scans.get().saturating_add(1));
        visit_grid(self, sheet_index, columns, rows, chunk_rows, visitor)
    }
}

#[cfg(test)]
impl StreamingSpreadsheetSession {
    pub(in crate::multi_format) fn reset_filter_grid_scan_count(&self) {
        self.filter_grid_scans.set(0);
    }

    pub(in crate::multi_format) fn filter_grid_scan_count(&self) -> usize {
        self.filter_grid_scans.get()
    }
}

fn visit_grid(
    session: &StreamingSpreadsheetSession,
    sheet_index: usize,
    columns: &[usize],
    rows: std::ops::Range<usize>,
    chunk_rows: usize,
    visitor: &mut FilterGridVisitor<'_>,
) -> Result<(), SpreadsheetEngineError> {
    if rows.is_empty() || columns.is_empty() {
        return visit_empty_filter_grid(rows, chunk_rows, visitor);
    }
    source::read_grid(session, sheet_index, columns, rows, chunk_rows, visitor)
}

fn visit_empty_filter_grid(
    rows: std::ops::Range<usize>,
    chunk_rows: usize,
    visitor: &mut FilterGridVisitor<'_>,
) -> Result<(), SpreadsheetEngineError> {
    for start in (rows.start..rows.end).step_by(chunk_rows) {
        let end = start.saturating_add(chunk_rows).min(rows.end);
        visitor(start..end, Vec::new())?;
    }
    Ok(())
}

struct StreamingFilterGridReader<'a, 'visitor> {
    columns: &'a [usize],
    rows: std::ops::Range<usize>,
    chunk_rows: usize,
    shared_strings: &'a [String],
    visitor: &'visitor mut FilterGridVisitor<'visitor>,
    next_chunk_start: usize,
    chunk_cells: Vec<SpreadsheetCellArtifact>,
    current_row: usize,
    current: Option<CellAccumulator>,
    capture: Capture,
}

impl<'a, 'visitor> StreamingFilterGridReader<'a, 'visitor> {
    fn new(
        columns: &'a [usize],
        rows: std::ops::Range<usize>,
        chunk_rows: usize,
        shared_strings: &'a [String],
        visitor: &'visitor mut FilterGridVisitor<'visitor>,
    ) -> Self {
        Self {
            columns,
            next_chunk_start: rows.start,
            rows,
            chunk_rows,
            shared_strings,
            visitor,
            chunk_cells: Vec::new(),
            current_row: 0,
            current: None,
            capture: Capture::None,
        }
    }

    fn finish(mut self) -> Result<(), SpreadsheetEngineError> {
        self.flush_before(self.rows.end)
    }

    fn flush_before(&mut self, row: usize) -> Result<(), SpreadsheetEngineError> {
        while self.next_chunk_start < self.rows.end && self.chunk_end() <= row {
            self.emit_chunk()?;
        }
        Ok(())
    }

    fn emit_chunk(&mut self) -> Result<(), SpreadsheetEngineError> {
        let start = self.next_chunk_start;
        let end = self.chunk_end();
        self.ensure_chunk_cells();
        let cells = std::mem::take(&mut self.chunk_cells);
        (self.visitor)(start..end, cells)?;
        self.next_chunk_start = end;
        Ok(())
    }

    fn chunk_end(&self) -> usize {
        self.next_chunk_start
            .saturating_add(self.chunk_rows)
            .min(self.rows.end)
    }

    fn ensure_chunk_cells(&mut self) {
        if self.chunk_cells.is_empty() {
            self.chunk_cells =
                filter_grid_cells(self.columns, self.next_chunk_start..self.chunk_end());
        }
    }

    fn finish_cell(&mut self) {
        let Some(cell) = self.current.take() else {
            return;
        };
        let index = cell.result_index();
        self.chunk_cells[index] = cell.finish(self.shared_strings);
    }
}

fn filter_grid_cells(
    columns: &[usize],
    rows: std::ops::Range<usize>,
) -> Vec<SpreadsheetCellArtifact> {
    rows.flat_map(|row| {
        columns
            .iter()
            .map(move |column| empty_cell(SpreadsheetCoordinate::new(row, *column)))
    })
    .collect()
}
