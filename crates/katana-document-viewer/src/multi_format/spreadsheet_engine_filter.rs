use super::SpreadsheetEngineSession;
use crate::multi_format::spreadsheet_engine::SpreadsheetEngineError;
use crate::multi_format::spreadsheet_filter_engine::SpreadsheetFilterResult;
use crate::multi_format::{SpreadsheetCellArtifact, SpreadsheetCoordinate};

impl SpreadsheetEngineSession {
    pub(crate) fn filter_candidates(
        &self,
        sheet_index: usize,
        column: usize,
        limit: usize,
    ) -> Result<(Vec<String>, bool), SpreadsheetEngineError> {
        crate::multi_format::spreadsheet_filter_engine::candidates(self, sheet_index, column, limit)
    }

    pub(crate) fn apply_filter(
        &mut self,
        sheet_index: usize,
        column: usize,
        values: Vec<String>,
    ) -> Result<SpreadsheetFilterResult, SpreadsheetEngineError> {
        let mut active = std::mem::take(&mut self.active_filters);
        let result = crate::multi_format::spreadsheet_filter_engine::apply(
            self,
            &mut active,
            sheet_index,
            column,
            values,
        );
        self.active_filters = active;
        result
    }

    pub(crate) fn clear_filter(
        &mut self,
        sheet_index: usize,
        column: Option<usize>,
    ) -> Result<SpreadsheetFilterResult, SpreadsheetEngineError> {
        let mut active = std::mem::take(&mut self.active_filters);
        let result = crate::multi_format::spreadsheet_filter_engine::clear(
            self,
            &mut active,
            sheet_index,
            column,
        );
        self.active_filters = active;
        result
    }

    pub(crate) fn materialize_filter_column(
        &self,
        sheet_index: usize,
        column: usize,
        rows: std::ops::Range<usize>,
    ) -> Result<Vec<SpreadsheetCellArtifact>, SpreadsheetEngineError> {
        self.materialize_filter_grid(sheet_index, &[column], rows)
    }

    pub(crate) fn materialize_filter_grid(
        &self,
        sheet_index: usize,
        columns: &[usize],
        rows: std::ops::Range<usize>,
    ) -> Result<Vec<SpreadsheetCellArtifact>, SpreadsheetEngineError> {
        let chunk_rows = (self.limits.max_materialized_cells / columns.len().max(1)).max(1);
        let mut cells = Vec::new();
        for start in (rows.start..rows.end).step_by(chunk_rows) {
            let end = start.saturating_add(chunk_rows).min(rows.end);
            let coordinates = filter_coordinates(columns, start..end);
            cells.extend(self.materialize(sheet_index, &coordinates)?);
        }
        Ok(cells)
    }
}

fn filter_coordinates(
    columns: &[usize],
    rows: std::ops::Range<usize>,
) -> Vec<SpreadsheetCoordinate> {
    rows.flat_map(|row| {
        columns
            .iter()
            .map(move |column| SpreadsheetCoordinate::new(row, *column))
    })
    .collect()
}
