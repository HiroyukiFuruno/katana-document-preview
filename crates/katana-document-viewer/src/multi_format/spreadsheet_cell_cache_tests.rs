use super::{MAX_CACHED_BYTES, MAX_CACHED_CELLS, SpreadsheetCellCache};
use crate::{
    SpreadsheetCellArtifact, SpreadsheetCellStyleArtifact, SpreadsheetCellValue,
    SpreadsheetConditionalFormattingArtifact, SpreadsheetCoordinate,
    SpreadsheetHorizontalAlignment, SpreadsheetVerticalAlignment,
};

#[test]
fn cache_resolves_in_request_order_and_evicts_by_entry_limit()
-> Result<(), Box<dyn std::error::Error>> {
    let mut cache = SpreadsheetCellCache::new();
    for row in 0..=MAX_CACHED_CELLS {
        cache.insert(0, cell(row, "value"));
    }
    assert_eq!(MAX_CACHED_CELLS, cache.len());
    assert_eq!(
        vec![SpreadsheetCoordinate::new(0, 0)],
        cache.missing(0, &[SpreadsheetCoordinate::new(0, 0)])
    );
    let coordinates = [
        SpreadsheetCoordinate::new(MAX_CACHED_CELLS, 0),
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
        },
        conditional_formatting: SpreadsheetConditionalFormattingArtifact::default(),
    }
}
