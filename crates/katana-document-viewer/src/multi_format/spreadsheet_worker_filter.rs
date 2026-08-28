use super::SpreadsheetWorkerLoop;
use crate::multi_format::spreadsheet_engine::SpreadsheetEngineError;
use crate::multi_format::spreadsheet_filter_engine::SpreadsheetFilterResult;
use crate::multi_format::spreadsheet_worker_protocol::SpreadsheetWorkerResponse;

impl SpreadsheetWorkerLoop {
    pub(super) fn filter_candidates(
        &mut self,
        request_id: u64,
        sheet_index: usize,
        column: usize,
        limit: usize,
    ) -> Result<(), String> {
        let _filter =
            crate::multi_format::debug_trace::DebugTrace::start("spreadsheet.filter_candidates");
        let response = match self.engine.filter_candidates(sheet_index, column, limit) {
            Ok((values, truncated)) => SpreadsheetWorkerResponse::FilterCandidates {
                request_id,
                sheet_index,
                column,
                values,
                truncated,
            },
            Err(error) => spreadsheet_failure(request_id, error),
        };
        self.write(&response)
    }

    pub(super) fn apply_filter(
        &mut self,
        request_id: u64,
        sheet_index: usize,
        column: usize,
        values: Vec<String>,
    ) -> Result<(), String> {
        let _filter =
            crate::multi_format::debug_trace::DebugTrace::start("spreadsheet.filter_apply");
        let response = match self.engine.apply_filter(sheet_index, column, values) {
            Ok(result) => filter_visibility_response(request_id, sheet_index, result),
            Err(error) => spreadsheet_failure(request_id, error),
        };
        self.write(&response)
    }

    pub(super) fn clear_filter(
        &mut self,
        request_id: u64,
        sheet_index: usize,
        column: Option<usize>,
    ) -> Result<(), String> {
        let _filter =
            crate::multi_format::debug_trace::DebugTrace::start("spreadsheet.filter_clear");
        let response = match self.engine.clear_filter(sheet_index, column) {
            Ok(result) => filter_visibility_response(request_id, sheet_index, result),
            Err(error) => spreadsheet_failure(request_id, error),
        };
        self.write(&response)
    }
}

fn filter_visibility_response(
    request_id: u64,
    sheet_index: usize,
    result: SpreadsheetFilterResult,
) -> SpreadsheetWorkerResponse {
    SpreadsheetWorkerResponse::FilterVisibility {
        request_id,
        sheet_index,
        applied_columns: result.applied_columns,
        visible_row_count: result.visible_row_count,
        filtered_out_rows: result.filtered_out_rows,
    }
}

fn spreadsheet_failure(
    request_id: u64,
    error: SpreadsheetEngineError,
) -> SpreadsheetWorkerResponse {
    SpreadsheetWorkerResponse::Failed {
        request_id: Some(request_id),
        stage: "spreadsheet".to_owned(),
        message: error.to_string(),
    }
}
