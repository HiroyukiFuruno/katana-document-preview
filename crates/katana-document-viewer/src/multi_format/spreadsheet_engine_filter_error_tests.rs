use super::{SpreadsheetEngineError, SpreadsheetEngineSession};
use crate::multi_format::SpreadsheetViewerLimits;
use crate::multi_format::spreadsheet_filter_test_support::representative_with_auto_filter;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn filter_evaluation_rejects_missing_active_sheet_state() -> TestResult {
    let engine = open_engine("missing-active-filter.xlsx")?;
    let active = Vec::new();
    assert!(matches!(
        crate::multi_format::spreadsheet_filter_engine::evaluate(&engine, &active, 0),
        Err(SpreadsheetEngineError::SheetOutsideDocument { .. })
    ));
    Ok(())
}

#[test]
fn filter_internal_operations_preserve_typed_error_paths() -> TestResult {
    let engine = open_engine("filter-errors.xlsx")?;
    assert_missing_active_errors(&engine);
    assert_invalid_filter_errors(&engine);
    assert_candidate_materialization_error()?;
    Ok(())
}

fn assert_missing_active_errors(engine: &SpreadsheetEngineSession) {
    let mut missing_active = Vec::new();
    assert!(matches!(
        crate::multi_format::spreadsheet_filter_engine::apply(
            engine,
            &mut missing_active,
            0,
            0,
            vec!["North".to_owned()],
        ),
        Err(SpreadsheetEngineError::SheetOutsideDocument { .. })
    ));
    assert!(matches!(
        crate::multi_format::spreadsheet_filter_engine::clear(engine, &mut missing_active, 0, None),
        Err(SpreadsheetEngineError::SheetOutsideDocument { .. })
    ));
}

fn assert_invalid_filter_errors(engine: &SpreadsheetEngineSession) {
    let mut active =
        crate::multi_format::spreadsheet_filter_engine::persisted_filters(engine.sheets());
    assert!(matches!(
        crate::multi_format::spreadsheet_filter_engine::clear(
            engine,
            &mut active,
            0,
            Some(usize::MAX),
        ),
        Err(SpreadsheetEngineError::FilterColumnOutsideRange { .. })
    ));
    assert!(matches!(
        crate::multi_format::spreadsheet_filter_engine::evaluate(engine, &active, usize::MAX),
        Err(SpreadsheetEngineError::SheetOutsideDocument { .. })
    ));
}

fn assert_candidate_materialization_error() -> TestResult {
    let mut malformed = open_engine("filter-materialization-error.xlsx")?;
    let invalid_column = malformed.sheets[0].column_count;
    malformed.sheets[0]
        .auto_filter
        .as_mut()
        .ok_or("auto filter is missing")?
        .range
        .end
        .column = invalid_column;
    assert!(matches!(
        malformed.filter_candidates(0, invalid_column, 1),
        Err(SpreadsheetEngineError::CellOutsideSheet { .. })
    ));
    Ok(())
}

fn open_engine(name: &str) -> Result<SpreadsheetEngineSession, Box<dyn std::error::Error>> {
    SpreadsheetEngineSession::open(
        representative_with_auto_filter()?,
        name,
        SpreadsheetViewerLimits::strict(),
    )
    .map_err(Into::into)
}
