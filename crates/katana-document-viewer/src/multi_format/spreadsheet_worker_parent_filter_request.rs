use crate::multi_format::SpreadsheetFilterCommand;
use crate::multi_format::spreadsheet_worker_protocol::SpreadsheetWorkerRequest;

pub(super) fn filter_request(
    request_id: u64,
    command: &SpreadsheetFilterCommand,
) -> SpreadsheetWorkerRequest {
    match command {
        SpreadsheetFilterCommand::Candidates {
            sheet_index,
            column,
            limit,
        } => candidate_request(request_id, *sheet_index, *column, *limit),
        SpreadsheetFilterCommand::ApplyValues {
            sheet_index,
            column,
            values,
        } => apply_request(request_id, *sheet_index, *column, values),
        SpreadsheetFilterCommand::Clear {
            sheet_index,
            column,
        } => SpreadsheetWorkerRequest::ClearFilter {
            request_id,
            sheet_index: *sheet_index,
            column: *column,
        },
    }
}

const fn candidate_request(
    request_id: u64,
    sheet_index: usize,
    column: usize,
    limit: usize,
) -> SpreadsheetWorkerRequest {
    SpreadsheetWorkerRequest::FilterCandidates {
        request_id,
        sheet_index,
        column,
        limit,
    }
}

fn apply_request(
    request_id: u64,
    sheet_index: usize,
    column: usize,
    values: &[String],
) -> SpreadsheetWorkerRequest {
    SpreadsheetWorkerRequest::ApplyFilter {
        request_id,
        sheet_index,
        column,
        values: values.to_vec(),
    }
}
