use super::{SpreadsheetWorkerRequest, SpreadsheetWorkerResponse};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn filter_requests_round_trip_without_cell_payloads() -> TestResult {
    let requests = [
        SpreadsheetWorkerRequest::FilterCandidates {
            request_id: 1,
            sheet_index: 0,
            column: 2,
            limit: 32,
        },
        SpreadsheetWorkerRequest::ApplyFilter {
            request_id: 2,
            sheet_index: 0,
            column: 2,
            values: vec!["Open".to_owned()],
        },
        SpreadsheetWorkerRequest::ClearFilter {
            request_id: 3,
            sheet_index: 0,
            column: None,
        },
    ];
    for request in requests {
        let encoded = serde_json::to_vec(&request)?;
        let decoded = serde_json::from_slice(&encoded)?;
        assert_eq!(request, decoded);
    }
    Ok(())
}

#[test]
fn filter_responses_return_only_candidates_or_row_visibility() -> TestResult {
    let responses = [
        SpreadsheetWorkerResponse::FilterCandidates {
            request_id: 1,
            sheet_index: 0,
            column: 2,
            values: vec!["Open".to_owned(), "Closed".to_owned()],
            truncated: false,
        },
        SpreadsheetWorkerResponse::FilterVisibility {
            request_id: 2,
            sheet_index: 0,
            applied_columns: vec![2],
            visible_row_count: 4,
            filtered_out_rows: vec![3, 5],
        },
    ];
    for response in responses {
        let encoded = serde_json::to_vec(&response)?;
        assert!(!String::from_utf8_lossy(&encoded).contains("cells"));
        let decoded = serde_json::from_slice(&encoded)?;
        assert_eq!(response, decoded);
    }
    Ok(())
}
