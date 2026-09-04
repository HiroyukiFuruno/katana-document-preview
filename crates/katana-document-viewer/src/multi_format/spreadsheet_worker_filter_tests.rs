use super::truncate_candidate_values;
use crate::multi_format::spreadsheet_worker_protocol::SpreadsheetWorkerResponse;

#[test]
fn candidate_truncation_stays_below_the_response_byte_limit()
-> Result<(), Box<dyn std::error::Error>> {
    let first = "first".repeat(32);
    let second = "second".repeat(32);
    let max_bytes = candidate_response_bytes(vec![first.clone()], false)?;
    let (values, truncated) =
        truncate_candidate_values(1, 0, 0, vec![first.clone(), second], false, max_bytes);

    assert_eq!(vec![first], values);
    assert!(truncated);
    assert!(candidate_response_bytes(values, truncated)? <= max_bytes);
    Ok(())
}

#[test]
fn candidate_truncation_counts_escaped_and_multibyte_json_values()
-> Result<(), Box<dyn std::error::Error>> {
    let first = "plain".to_owned();
    let second = "quote\"\\\u{0007}\n日本".repeat(16);
    let values = vec![first.clone(), second];
    let max_bytes = candidate_response_bytes(values.clone(), false)?;
    let (accepted, truncated) =
        truncate_candidate_values(1, 0, 0, values.clone(), false, max_bytes);

    assert_eq!(values, accepted);
    assert!(!truncated);
    let (accepted, truncated) =
        truncate_candidate_values(1, 0, 0, values, false, max_bytes.saturating_sub(1));
    assert_eq!(vec![first], accepted);
    assert!(truncated);
    assert!(candidate_response_bytes(accepted, truncated)? <= max_bytes.saturating_sub(1));
    Ok(())
}

fn candidate_response_bytes(
    values: Vec<String>,
    truncated: bool,
) -> Result<usize, serde_json::Error> {
    serde_json::to_vec(&SpreadsheetWorkerResponse::FilterCandidates {
        request_id: 1,
        sheet_index: 0,
        column: 0,
        values,
        truncated,
    })
    .map(|bytes| bytes.len())
}
