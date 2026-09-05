use super::{SpreadsheetEngineError, SpreadsheetEngineSession};
use crate::multi_format::spreadsheet_filter_test_support::{
    representative_with_auto_filter, representative_with_persisted_auto_filter_and_authored_hidden,
    representative_with_persisted_auto_filter_and_saved_filter_hidden_rows,
};
use crate::multi_format::{SpreadsheetFilterCriterion, SpreadsheetViewerLimits};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn opening_persisted_auto_filter_keeps_criteria_and_clear_restores_filtered_rows() -> TestResult {
    let mut engine = SpreadsheetEngineSession::open(
        representative_with_persisted_auto_filter_and_saved_filter_hidden_rows()?,
        "persisted-filter.xlsx",
        SpreadsheetViewerLimits::strict(),
    )?;
    let filter = engine.sheets()[0]
        .auto_filter
        .as_ref()
        .ok_or("auto filter is missing")?;
    assert_eq!(vec![4, 5, 6], filter.filtered_out_rows);
    assert!(
        engine.sheets()[0].row_tracks[4..=6]
            .iter()
            .all(|track| !track.hidden)
    );

    let cleared = engine.clear_filter(0, None)?;
    assert!(cleared.applied_columns.is_empty());
    assert!(cleared.filtered_out_rows.is_empty());
    assert_eq!(7, cleared.visible_row_count);
    Ok(())
}

#[test]
fn clear_persisted_filter_preserves_authored_hidden_rows() -> TestResult {
    let mut engine = SpreadsheetEngineSession::open(
        representative_with_persisted_auto_filter_and_authored_hidden()?,
        "persisted-filter-with-authored-hidden.xlsx",
        SpreadsheetViewerLimits::strict(),
    )?;
    let filter = engine.sheets()[0]
        .auto_filter
        .as_ref()
        .ok_or("auto filter is missing")?;
    assert_eq!(vec![4, 5, 6], filter.filtered_out_rows);
    assert!(engine.sheets()[0].row_tracks[3].hidden);
    assert!(!engine.sheets()[0].row_tracks[4].hidden);

    let cleared = engine.clear_filter(0, None)?;
    assert_eq!(6, cleared.visible_row_count);
    assert!(engine.sheets()[0].row_tracks[3].hidden);
    assert!(!engine.sheets()[0].row_tracks[4].hidden);
    Ok(())
}

#[test]
fn persisted_filter_initialization_fails_closed_for_invalid_materialization() -> TestResult {
    let mut engine = open_engine("invalid-persisted-filter.xlsx")?;
    let sheet = &mut engine.sheets[0];
    let filter = sheet.auto_filter.as_mut().ok_or("auto filter is missing")?;
    filter.columns[0].column = sheet.column_count;

    assert!(matches!(
        engine.initialize_persisted_filters(),
        Err(SpreadsheetEngineError::CellOutsideSheet { .. })
    ));
    Ok(())
}

#[test]
fn persisted_filter_skips_criteria_that_cannot_be_replayed_as_selected_values() -> TestResult {
    let mut engine = open_engine("persisted-criteria.xlsx")?;
    set_criteria(&mut engine, SpreadsheetFilterCriterion::Blank)?;
    let active = crate::multi_format::spreadsheet_filter_engine::persisted_filters(engine.sheets());
    assert!(active[0][&0].contains(""));

    for criterion in [
        SpreadsheetFilterCriterion::NonBlank,
        SpreadsheetFilterCriterion::Unsupported("customFilters".to_owned()),
    ] {
        set_criteria(&mut engine, criterion)?;
        assert!(
            crate::multi_format::spreadsheet_filter_engine::persisted_filters(engine.sheets())[0]
                .is_empty()
        );
    }
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

fn set_criteria(
    engine: &mut SpreadsheetEngineSession,
    criterion: SpreadsheetFilterCriterion,
) -> TestResult {
    engine.sheets[0]
        .auto_filter
        .as_mut()
        .ok_or("auto filter is missing")?
        .columns[0]
        .criteria = vec![criterion];
    Ok(())
}
