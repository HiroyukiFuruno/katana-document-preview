use super::{SpreadsheetEngineError, SpreadsheetEngineSession};
use crate::multi_format::spreadsheet_filter_test_support::{
    representative_with_auto_filter, representative_with_auto_filter_and_blank,
};
use crate::multi_format::{SpreadsheetCoordinate, SpreadsheetViewerLimits};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[path = "spreadsheet_engine_filter_streaming_tests.rs"]
mod streaming_tests;

#[path = "spreadsheet_engine_filter_initialization_tests.rs"]
mod initialization_tests;

#[path = "spreadsheet_engine_filter_grid_tests.rs"]
mod grid_tests;

#[test]
fn candidates_apply_and_clear_preserve_original_row_indices() -> TestResult {
    let mut engine = SpreadsheetEngineSession::open(
        representative_with_auto_filter()?,
        "filter.xlsx",
        SpreadsheetViewerLimits::strict(),
    )?;
    let (candidates, truncated) = engine.filter_candidates(0, 0, 16)?;
    assert_eq!(vec!["East", "North", "South", "West"], candidates);
    assert!(!truncated);

    let applied = engine.apply_filter(0, 0, vec!["North".to_owned()])?;
    assert_eq!(vec![0], applied.applied_columns);
    assert_eq!(vec![4, 5, 6], applied.filtered_out_rows);
    assert_eq!(4, applied.visible_row_count);

    let combined = engine.apply_filter(0, 1, vec!["98".to_owned()])?;
    assert_eq!(vec![0, 1], combined.applied_columns);
    assert_eq!(vec![3, 4, 5, 6], combined.filtered_out_rows);
    let remaining = engine.clear_filter(0, Some(0))?;
    assert_eq!(vec![1], remaining.applied_columns);
    assert_eq!(vec![3, 5, 6], remaining.filtered_out_rows);

    let cleared = engine.clear_filter(0, None)?;
    assert!(cleared.applied_columns.is_empty());
    assert!(cleared.filtered_out_rows.is_empty());
    assert_eq!(7, cleared.visible_row_count);
    Ok(())
}

#[test]
fn filter_limits_and_authored_hidden_rows_remain_enforced() -> TestResult {
    let mut engine = SpreadsheetEngineSession::open(
        representative_with_auto_filter()?,
        "filter.xlsx",
        SpreadsheetViewerLimits::strict(),
    )?;
    engine.sheets[0].row_tracks[1].hidden = true;
    let (candidates, truncated) = engine.filter_candidates(0, 0, 2)?;
    assert_eq!(2, candidates.len());
    assert!(truncated);
    assert!(engine.filter_candidates(0, 0, 0).is_err());
    assert!(engine.filter_candidates(0, 7, 2).is_err());

    let applied = engine.apply_filter(0, 0, vec!["North".to_owned()])?;
    assert_eq!(3, applied.visible_row_count);
    assert!(engine.clear_filter(1, None).is_err());
    Ok(())
}

#[test]
fn filter_accepts_blank_and_multiple_values_from_the_real_xlsx_fixture() -> TestResult {
    let mut engine = SpreadsheetEngineSession::open(
        representative_with_auto_filter_and_blank()?,
        "filter-blank.xlsx",
        SpreadsheetViewerLimits::strict(),
    )?;
    let (candidates, truncated) = engine.filter_candidates(0, 0, 16)?;
    assert_eq!(vec!["", "East", "North", "South"], candidates);
    assert!(!truncated);

    let applied = engine.apply_filter(0, 0, vec!["".to_owned(), "North".to_owned()])?;
    assert_eq!(vec![4, 5], applied.filtered_out_rows);
    assert_eq!(5, applied.visible_row_count);

    let cleared = engine.clear_filter(0, None)?;
    assert!(cleared.applied_columns.is_empty());
    assert!(cleared.filtered_out_rows.is_empty());
    assert_eq!(7, cleared.visible_row_count);
    Ok(())
}

#[test]
fn filter_evaluation_chunks_ranges_larger_than_materialization_limit() -> TestResult {
    let limits = SpreadsheetViewerLimits {
        max_sheets: 256,
        max_logical_cells: 25_000_000,
        max_materialized_cells: 2,
    };
    let mut engine =
        SpreadsheetEngineSession::open(representative_with_auto_filter()?, "filter.xlsx", limits)?;
    let (candidates, truncated) = engine.filter_candidates(0, 0, 16)?;
    assert_eq!(4, candidates.len());
    assert!(!truncated);
    let applied = engine.apply_filter(0, 0, vec!["West".to_owned()])?;
    assert_eq!(vec![3, 4, 5], applied.filtered_out_rows);
    Ok(())
}

#[test]
fn invalid_filter_and_materialization_requests_fail_closed() -> TestResult {
    let mut engine = SpreadsheetEngineSession::open(
        representative_with_auto_filter()?,
        "filter.xlsx",
        SpreadsheetViewerLimits::strict(),
    )?;
    assert!(matches!(
        engine.apply_filter(0, 0, vec!["x".to_owned(); 4_097]),
        Err(SpreadsheetEngineError::FilterValueLimit { .. })
    ));
    assert!(matches!(
        engine.materialize(0, &[SpreadsheetCoordinate::new(usize::MAX, 0)]),
        Err(SpreadsheetEngineError::CellOutsideSheet { .. })
    ));
    let duplicate = SpreadsheetCoordinate::new(0, 0);
    assert!(matches!(
        engine.materialize(0, &[duplicate, duplicate]),
        Err(SpreadsheetEngineError::DuplicateCell { .. })
    ));
    let too_many = vec![SpreadsheetCoordinate::new(0, 0); 4_097];
    assert!(matches!(
        engine.materialize(0, &too_many),
        Err(SpreadsheetEngineError::ResourceLimit { .. })
    ));
    Ok(())
}

#[test]
fn filter_helpers_cover_missing_cells_and_absent_metadata() -> TestResult {
    let engine = SpreadsheetEngineSession::open(
        representative_with_auto_filter()?,
        "filter.xlsx",
        SpreadsheetViewerLimits::strict(),
    )?;
    let rows = 1..3;
    let mut selected = std::collections::BTreeSet::new();
    selected.insert("North".to_owned());
    let mut filters = std::collections::BTreeMap::new();
    filters.insert(0, selected);
    assert_eq!(
        vec![1, 2],
        crate::multi_format::spreadsheet_filter_engine::rejected_rows(
            rows,
            &[0],
            &filters,
            Vec::new(),
        )
    );

    assert_plain_sheet_has_no_filter()?;
    assert!(!engine.sheets().is_empty());
    Ok(())
}

fn assert_plain_sheet_has_no_filter() -> TestResult {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/multi-format/representative.xlsx");
    let plain = SpreadsheetEngineSession::open(
        std::fs::read(fixture)?,
        "plain.xlsx",
        SpreadsheetViewerLimits::strict(),
    )?;
    assert!(matches!(
        plain.filter_candidates(0, 0, 8),
        Err(SpreadsheetEngineError::FilterUnavailable { .. })
    ));
    assert_eq!(
        0..0,
        crate::multi_format::spreadsheet_filter_engine::filter_rows(plain.sheet(0)?)
    );
    Ok(())
}
