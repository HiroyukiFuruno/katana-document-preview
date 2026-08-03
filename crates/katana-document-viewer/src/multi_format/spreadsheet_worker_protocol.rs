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
    Failed {
        request_id: Option<u64>,
        stage: String,
        message: String,
    },
    Stopped,
}
