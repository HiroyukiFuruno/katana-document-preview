use super::*;

#[test]
fn persisted_filter_tolerates_short_row_track_metadata() -> TestResult {
    let mut engine = SpreadsheetEngineSession::open(
        representative_with_auto_filter()?,
        "filter.xlsx",
        SpreadsheetViewerLimits::strict(),
    )?;
    engine.sheets[0].row_tracks.truncate(1);

    engine.initialize_persisted_filters()?;

    let filter = engine.sheets()[0]
        .auto_filter
        .as_ref()
        .ok_or("auto filter is missing")?;
    assert_eq!(vec![4, 5, 6], filter.filtered_out_rows);
    Ok(())
}
