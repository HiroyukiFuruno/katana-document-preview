use super::{MAX_CACHED_BYTES, SpreadsheetCellCache};
use crate::{
    SpreadsheetCellArtifact, SpreadsheetCellBorderArtifact, SpreadsheetCellStyleArtifact,
    SpreadsheetCellValue, SpreadsheetConditionalFormattingArtifact, SpreadsheetCoordinate,
    SpreadsheetHorizontalAlignment, SpreadsheetVerticalAlignment,
};

#[test]
fn cache_resolves_in_request_order_and_evicts_by_entry_limit()
-> Result<(), Box<dyn std::error::Error>> {
    let max_cells = 3;
    let mut cache = SpreadsheetCellCache::with_limits(max_cells, MAX_CACHED_BYTES);
    for row in 0..=max_cells {
        cache.insert(0, cell(row, "value"));
    }
    assert_eq!(max_cells, cache.len());
    assert_eq!(
        vec![SpreadsheetCoordinate::new(0, 0)],
        cache.missing(0, &[SpreadsheetCoordinate::new(0, 0)])
    );
    let coordinates = [
        SpreadsheetCoordinate::new(max_cells, 0),
        SpreadsheetCoordinate::new(1, 0),
    ];
    let resolved = cache.resolve(0, &coordinates)?;
    let resolved_coordinates = resolved
        .iter()
        .map(|cell| cell.coordinate)
        .collect::<Vec<_>>();
    assert_eq!(coordinates.as_slice(), resolved_coordinates);
    Ok(())
}

#[test]
fn cache_rejects_oversized_cells_and_reports_missing_values() {
    let mut cache = SpreadsheetCellCache::new();
    cache.insert(0, cell(1, "first"));
    cache.insert(0, cell(1, "replacement"));
    assert_eq!(1, cache.len());
    cache.insert(0, cell(0, &"x".repeat(MAX_CACHED_BYTES)));
    assert_eq!(1, cache.len());
    assert!(cache.byte_count() > 0);
    assert!(
        cache
            .resolve(0, &[SpreadsheetCoordinate::new(0, 0)])
            .is_err()
    );
}

#[test]
fn current_materialization_response_survives_cache_rejection()
-> Result<(), Box<dyn std::error::Error>> {
    let mut cache = SpreadsheetCellCache::with_limits(1, 0);
    let coordinates = [
        SpreadsheetCoordinate::new(0, 0),
        SpreadsheetCoordinate::new(1, 0),
    ];
    let resolved =
        cache.resolve_materialized(0, &coordinates, vec![cell(0, "first"), cell(1, "second")])?;

    assert_eq!(
        coordinates.as_slice(),
        resolved
            .iter()
            .map(|cell| cell.coordinate)
            .collect::<Vec<_>>()
    );
    assert_eq!(0, cache.len());
    Ok(())
}

#[test]
fn oversized_current_materialization_response_is_returned_without_cache_admission()
-> Result<(), Box<dyn std::error::Error>> {
    let mut cache = SpreadsheetCellCache::new();
    let coordinates = [SpreadsheetCoordinate::new(0, 0)];
    let oversized = "x".repeat(MAX_CACHED_BYTES);
    let resolved = cache.resolve_materialized(0, &coordinates, vec![cell(0, &oversized)])?;

    assert_eq!(MAX_CACHED_BYTES, resolved[0].display_text.len());
    assert_eq!(0, cache.len());
    Ok(())
}

#[test]
fn current_materialization_response_preserves_existing_request_hits_after_eviction()
-> Result<(), Box<dyn std::error::Error>> {
    let mut cache = SpreadsheetCellCache::with_limits(1, MAX_CACHED_BYTES);
    cache.insert(0, cell(0, "cached"));
    let coordinates = [
        SpreadsheetCoordinate::new(0, 0),
        SpreadsheetCoordinate::new(1, 0),
    ];
    let resolved = cache.resolve_materialized(0, &coordinates, vec![cell(1, "fresh")])?;

    assert_eq!(
        vec!["cached", "fresh"],
        resolved
            .iter()
            .map(|cell| cell.display_text.as_str())
            .collect::<Vec<_>>()
    );
    assert_eq!(1, cache.len());
    Ok(())
}

#[test]
fn materialization_cache_evicts_in_request_order() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = SpreadsheetCellCache::with_limits(1, MAX_CACHED_BYTES);
    let coordinates = [
        SpreadsheetCoordinate::new(0, 0),
        SpreadsheetCoordinate::new(1, 0),
    ];
    cache.resolve_materialized(0, &coordinates, vec![cell(0, "first"), cell(1, "second")])?;

    assert!(cache.resolve(0, &[coordinates[0]]).is_err());
    assert_eq!(
        "second",
        cache.resolve(0, &[coordinates[1]])?[0].display_text
    );
    Ok(())
}

#[test]
fn materialization_response_rejects_unrequested_duplicate_and_missing_cells() {
    let coordinate = SpreadsheetCoordinate::new(0, 0);
    let unrequested = SpreadsheetCellCache::new().resolve_materialized(
        0,
        &[coordinate],
        vec![cell(1, "unrequested")],
    );
    assert!(matches!(
        unrequested,
        Err(crate::OfficeWorkerError::Protocol { .. })
    ));

    let duplicate = SpreadsheetCellCache::new().resolve_materialized(
        0,
        &[coordinate],
        vec![cell(0, "first"), cell(0, "second")],
    );
    assert!(matches!(
        duplicate,
        Err(crate::OfficeWorkerError::Protocol { .. })
    ));

    let missing = SpreadsheetCellCache::new().resolve_materialized(0, &[coordinate], Vec::new());
    assert!(matches!(
        missing,
        Err(crate::OfficeWorkerError::Protocol { .. })
    ));
}

fn cell(row: usize, text: &str) -> SpreadsheetCellArtifact {
    SpreadsheetCellArtifact {
        coordinate: SpreadsheetCoordinate::new(row, 0),
        display_text: text.to_owned(),
        value: SpreadsheetCellValue::Text(text.to_owned()),
        formula: None,
        style: SpreadsheetCellStyleArtifact {
            font_name: "Aptos".to_owned(),
            font_size: 11.0,
            font_color: None,
            fill_color: None,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            horizontal_alignment: SpreadsheetHorizontalAlignment::General,
            vertical_alignment: SpreadsheetVerticalAlignment::Bottom,
            wrap_text: false,
            number_format: "General".to_owned(),
            borders: SpreadsheetCellBorderArtifact::default(),
        },
        conditional_formatting: SpreadsheetConditionalFormattingArtifact::default(),
    }
}
