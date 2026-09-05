use super::SpreadsheetEngineSession;
use crate::multi_format::spreadsheet_engine::SpreadsheetEngineError;
use crate::multi_format::spreadsheet_filter_engine::SpreadsheetFilterResult;
use crate::multi_format::{SpreadsheetCellArtifact, SpreadsheetCoordinate};

type FilterGridVisitor<'a> = dyn FnMut(
        std::ops::Range<usize>,
        Vec<SpreadsheetCellArtifact>,
    ) -> Result<(), SpreadsheetEngineError>
    + 'a;

impl SpreadsheetEngineSession {
    pub(super) fn initialize_persisted_filters(&mut self) -> Result<(), SpreadsheetEngineError> {
        let active =
            crate::multi_format::spreadsheet_filter_engine::persisted_filters(&self.sheets);
        let mut initial_filtered_rows = Vec::with_capacity(self.sheets.len());
        for sheet_index in 0..self.sheets.len() {
            initial_filtered_rows.push(
                crate::multi_format::spreadsheet_filter_engine::evaluate(
                    self,
                    &active,
                    sheet_index,
                )?
                .filtered_out_rows,
            );
        }
        for (sheet, filtered_out_rows) in self.sheets.iter_mut().zip(initial_filtered_rows) {
            if let Some(filter) = &mut sheet.auto_filter {
                filter.filtered_out_rows = filtered_out_rows.clone();
            }
            move_persisted_filter_hidden_rows_to_filter_layer(sheet, &filtered_out_rows);
        }
        self.active_filters = active;
        Ok(())
    }

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

    pub(crate) fn visit_filter_grid(
        &self,
        sheet_index: usize,
        columns: &[usize],
        rows: std::ops::Range<usize>,
        visitor: &mut FilterGridVisitor<'_>,
    ) -> Result<(), SpreadsheetEngineError> {
        let chunk_rows = (self.limits.max_materialized_cells / columns.len().max(1)).max(1);
        if !rows.is_empty() {
            self.validate_request(
                sheet_index,
                &filter_coordinates(columns, rows.start..rows.start.saturating_add(1)),
            )?;
        }
        if let super::SpreadsheetEngineBackend::Streaming(streaming) = &self.backend {
            return streaming.visit_filter_grid(sheet_index, columns, rows, chunk_rows, visitor);
        }
        for start in (rows.start..rows.end).step_by(chunk_rows) {
            let end = start.saturating_add(chunk_rows).min(rows.end);
            let coordinates = filter_coordinates(columns, start..end);
            visitor(start..end, self.materialize(sheet_index, &coordinates)?)?;
        }
        Ok(())
    }
}

fn move_persisted_filter_hidden_rows_to_filter_layer(
    sheet: &mut crate::multi_format::SpreadsheetSheetArtifact,
    filtered_out_rows: &[usize],
) {
    // SpreadsheetML は手動非表示と保存済み AutoFilter に共通の row.hidden を使う。
    // 再生可能な保存条件で除外される行は filter layer を正とし、Clear が保存時の
    // visibility を残さず全行を復元できるようにする。条件外の source 非表示は
    // authored layer に残る。両者が重なる場合はファイル上で起源を区別できないため、
    // 保存済み filter の再生結果を採用する。
    for row in filtered_out_rows {
        if let Some(track) = sheet.row_tracks.get_mut(*row)
            && track.hidden
        {
            track.hidden = false;
        }
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
