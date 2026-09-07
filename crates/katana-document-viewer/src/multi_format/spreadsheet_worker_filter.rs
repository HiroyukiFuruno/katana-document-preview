use super::SpreadsheetWorkerLoop;
use crate::multi_format::spreadsheet_engine::SpreadsheetEngineError;
use crate::multi_format::spreadsheet_filter_engine::SpreadsheetFilterResult;
use crate::multi_format::spreadsheet_worker_protocol::{
    MAX_SPREADSHEET_REQUEST_BYTES, MAX_SPREADSHEET_RESPONSE_BYTES, SpreadsheetWorkerResponse,
};

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
            Ok((values, truncated)) => {
                candidate_response(request_id, sheet_index, column, values, truncated)
            }
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

fn candidate_response(
    request_id: u64,
    sheet_index: usize,
    column: usize,
    values: Vec<String>,
    truncated: bool,
) -> SpreadsheetWorkerResponse {
    let (values, truncated) = truncate_candidate_values(
        request_id,
        sheet_index,
        column,
        values,
        truncated,
        MAX_SPREADSHEET_RESPONSE_BYTES,
    );
    SpreadsheetWorkerResponse::FilterCandidates {
        request_id,
        sheet_index,
        column,
        values,
        truncated,
    }
}

fn truncate_candidate_values(
    request_id: u64,
    sheet_index: usize,
    column: usize,
    values: Vec<String>,
    truncated: bool,
    max_bytes: usize,
) -> (Vec<String>, bool) {
    let mut response_bytes =
        candidate_response_envelope_bytes(request_id, sheet_index, column, truncated);
    let mut apply_request_bytes =
        apply_filter_request_envelope_bytes(u64::MAX, sheet_index, column);
    let mut accepted = Vec::with_capacity(values.len());
    for value in values {
        let value_bytes = json_string_bytes(&value);
        let separator_bytes = usize::from(!accepted.is_empty());
        let candidate_bytes = next_json_array_bytes(response_bytes, separator_bytes, value_bytes);
        let candidate_apply_request_bytes =
            next_json_array_bytes(apply_request_bytes, separator_bytes, value_bytes);
        if !candidate_fits_limits(candidate_bytes, candidate_apply_request_bytes, max_bytes) {
            return (accepted, true);
        }
        response_bytes = candidate_bytes;
        apply_request_bytes = candidate_apply_request_bytes;
        accepted.push(value);
    }
    (accepted, truncated)
}

fn next_json_array_bytes(base: usize, separator: usize, value: usize) -> usize {
    base.saturating_add(separator).saturating_add(value)
}

fn candidate_fits_limits(candidate_bytes: usize, apply_bytes: usize, max_bytes: usize) -> bool {
    candidate_bytes <= max_bytes && apply_bytes <= MAX_SPREADSHEET_REQUEST_BYTES
}

fn apply_filter_request_envelope_bytes(
    request_id: u64,
    sheet_index: usize,
    column: usize,
) -> usize {
    r#"{"command":"apply_filter","request_id":"#
        .len()
        .saturating_add(request_id.to_string().len())
        .saturating_add(r#","sheet_index":"#.len())
        .saturating_add(sheet_index.to_string().len())
        .saturating_add(r#","column":"#.len())
        .saturating_add(column.to_string().len())
        .saturating_add(r#","values":["#.len())
        .saturating_add(2)
        .saturating_add(1)
}

fn candidate_response_envelope_bytes(
    request_id: u64,
    sheet_index: usize,
    column: usize,
    truncated: bool,
) -> usize {
    r#"{"status":"filter_candidates","request_id":"#
        .len()
        .saturating_add(request_id.to_string().len())
        .saturating_add(r#","sheet_index":"#.len())
        .saturating_add(sheet_index.to_string().len())
        .saturating_add(r#","column":"#.len())
        .saturating_add(column.to_string().len())
        .saturating_add(r#","values":["#.len())
        .saturating_add(r#"],"truncated":"#.len())
        .saturating_add(if truncated { "true}" } else { "false}" }.len())
}

fn json_string_bytes(value: &str) -> usize {
    value.bytes().fold(2, |bytes, byte| {
        bytes.saturating_add(match byte {
            b'"' | b'\\' | b'\x08' | b'\t' | b'\n' | b'\x0C' | b'\r' => 2,
            0..=0x1F => 6,
            _ => 1,
        })
    })
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

#[cfg(test)]
#[path = "spreadsheet_worker_filter_tests.rs"]
mod tests;
