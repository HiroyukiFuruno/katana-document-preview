use super::{SpreadsheetCellArtifact, SpreadsheetCoordinate, SpreadsheetSheetArtifact};
use serde::{Deserialize, Serialize};

pub(super) const SPREADSHEET_MODE: &str = "--spreadsheet";
pub(super) const MAX_SPREADSHEET_REQUEST_BYTES: usize = 512 * 1024;
pub(super) const MAX_SPREADSHEET_RESPONSE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub(super) enum SpreadsheetWorkerRequest {
    Materialize {
        request_id: u64,
        sheet_index: usize,
        coordinates: Vec<SpreadsheetCoordinate>,
    },
    FilterCandidates {
        request_id: u64,
        sheet_index: usize,
        column: usize,
        limit: usize,
    },
    ApplyFilter {
        request_id: u64,
        sheet_index: usize,
        column: usize,
        values: Vec<String>,
    },
    ClearFilter {
        request_id: u64,
        sheet_index: usize,
        column: Option<usize>,
    },
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(super) enum SpreadsheetWorkerResponse {
    Opened {
        sheets: Vec<SpreadsheetSheetArtifact>,
    },
    Materialized {
        request_id: u64,
        cells: Vec<SpreadsheetCellArtifact>,
    },
    FilterCandidates {
        request_id: u64,
        sheet_index: usize,
        column: usize,
        values: Vec<String>,
        truncated: bool,
    },
    FilterVisibility {
        request_id: u64,
        sheet_index: usize,
        applied_columns: Vec<usize>,
        visible_row_count: usize,
        filtered_out_rows: Vec<usize>,
    },
    Failed {
        request_id: Option<u64>,
        stage: String,
        message: String,
    },
    Stopped,
}

#[cfg(test)]
#[path = "spreadsheet_worker_protocol_tests.rs"]
mod tests;
