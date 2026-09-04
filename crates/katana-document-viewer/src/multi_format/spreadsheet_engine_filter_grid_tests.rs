use super::super::{SpreadsheetEngineError, SpreadsheetEngineSession};
use crate::multi_format::spreadsheet_filter_test_support::representative_with_auto_filter;
use crate::multi_format::{SpreadsheetCellArtifact, SpreadsheetViewerLimits};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn filter_grid_visits_one_bounded_materialization_chunk_at_a_time() -> TestResult {
    let limits = SpreadsheetViewerLimits {
        max_sheets: 256,
        max_logical_cells: 25_000_000,
        max_materialized_cells: 2,
    };
    let engine =
        SpreadsheetEngineSession::open(representative_with_auto_filter()?, "filter.xlsx", limits)?;
    let rows = crate::multi_format::spreadsheet_filter_engine::filter_rows(engine.sheet(0)?);
    let mut largest_chunk = 0;
    let mut count_chunk = |chunk_rows: std::ops::Range<usize>,
                           cells: Vec<SpreadsheetCellArtifact>| {
        largest_chunk = largest_chunk.max(cells.len());
        assert_eq!(chunk_rows.len() * 2, cells.len());
        Ok(())
    };
    engine.visit_filter_grid(0, &[0, 1], rows, &mut count_chunk)?;
    assert_eq!(2, largest_chunk);
    Ok(())
}

#[test]
fn filter_grid_propagates_model_visitor_errors() -> TestResult {
    let engine = SpreadsheetEngineSession::open(
        representative_with_auto_filter()?,
        "filter.xlsx",
        SpreadsheetViewerLimits::strict(),
    )?;
    let mut stop_visitor = |_: std::ops::Range<usize>, _: Vec<SpreadsheetCellArtifact>| {
        Err(SpreadsheetEngineError::Import(
            "stop model visitor".to_owned(),
        ))
    };

    let result = engine.visit_filter_grid(0, &[0], 0..1, &mut stop_visitor);

    assert!(matches!(
        result,
        Err(SpreadsheetEngineError::Import(message)) if message == "stop model visitor"
    ));
    Ok(())
}

#[test]
fn filter_grid_propagates_later_model_materialization_errors() -> TestResult {
    let engine = SpreadsheetEngineSession::open(
        representative_with_auto_filter()?,
        "filter.xlsx",
        SpreadsheetViewerLimits::strict(),
    )?;
    let mut accept_chunk = |_: std::ops::Range<usize>, _: Vec<SpreadsheetCellArtifact>| Ok(());

    let result = engine.visit_filter_grid(0, &[0], 0..8, &mut accept_chunk);

    assert!(matches!(
        result,
        Err(SpreadsheetEngineError::CellOutsideSheet {
            row: 7,
            column: 0,
            ..
        })
    ));
    Ok(())
}
