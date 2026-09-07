use super::{SpreadsheetCellCache, cell_bytes};
use crate::{
    SpreadsheetBorderSideArtifact, SpreadsheetCellArtifact, SpreadsheetCellBorderArtifact,
    SpreadsheetCellStyleArtifact, SpreadsheetCellValue, SpreadsheetConditionalFormattingArtifact,
    SpreadsheetCoordinate, SpreadsheetDataBarArtifact, SpreadsheetHorizontalAlignment,
    SpreadsheetIconArtifact, SpreadsheetRatingArtifact, SpreadsheetVerticalAlignment,
};

#[test]
fn cache_capacity_includes_all_owned_cell_heap_values() -> Result<(), Box<dyn std::error::Error>> {
    let cell = rich_cell(0);
    let bytes = cell_bytes(&cell);
    assert!(bytes > legacy_cell_bytes(&cell));

    let mut too_small = SpreadsheetCellCache::with_limits(1, bytes.saturating_sub(1));
    too_small.insert(0, cell.clone());
    assert_eq!(0, too_small.len());

    let mut exact = SpreadsheetCellCache::with_limits(1, bytes);
    exact.insert(0, cell.clone());
    assert_eq!(1, exact.len());
    assert_eq!(bytes, exact.byte_count());
    assert_eq!(
        cell.display_text,
        exact.resolve(0, &[SpreadsheetCoordinate::new(0, 0)])?[0].display_text
    );
    Ok(())
}

#[test]
fn cache_capacity_keeps_scalar_cell_values_inline() {
    let mut artifact = basic_cell(0, "value");
    for value in [
        SpreadsheetCellValue::Empty,
        SpreadsheetCellValue::Number(1.0),
        SpreadsheetCellValue::Boolean(true),
    ] {
        artifact.value = value;
        assert!(cell_bytes(&artifact) > 0);
    }
}

fn legacy_cell_bytes(cell: &SpreadsheetCellArtifact) -> usize {
    std::mem::size_of::<SpreadsheetCellArtifact>()
        .saturating_add(cell.display_text.len())
        .saturating_add(cell.formula.as_ref().map_or(0, String::len))
        .saturating_add(cell.style.font_name.len())
        .saturating_add(cell.style.number_format.len())
}

fn rich_cell(row: usize) -> SpreadsheetCellArtifact {
    let mut artifact = basic_cell(row, &"display".repeat(16));
    artifact.formula = Some("formula".repeat(16));
    configure_rich_style(&mut artifact);
    artifact.conditional_formatting = rich_conditional_formatting();
    artifact
}

fn configure_rich_style(artifact: &mut SpreadsheetCellArtifact) {
    artifact.style.font_name = "font-name".repeat(16);
    artifact.style.font_color = Some("font-color".repeat(16));
    artifact.style.fill_color = Some("fill-color".repeat(16));
    artifact.style.number_format = "number-format".repeat(16);
    artifact.style.borders = SpreadsheetCellBorderArtifact {
        left: Some(border_side("left")),
        right: Some(border_side("right")),
        top: Some(border_side("top")),
        bottom: Some(border_side("bottom")),
    };
}

fn rich_conditional_formatting() -> SpreadsheetConditionalFormattingArtifact {
    SpreadsheetConditionalFormattingArtifact {
        applied: true,
        data_bar: Some(SpreadsheetDataBarArtifact {
            positive_color: Some("positive".repeat(16)),
            negative_color: Some("negative".repeat(16)),
            value: 0.5,
            axis_position: 0.5,
            gradient: true,
            show_value: true,
        }),
        icon: Some(SpreadsheetIconArtifact {
            name: "icon".repeat(16),
            color: Some("icon-color".repeat(16)),
            show_value: true,
        }),
        rating: Some(SpreadsheetRatingArtifact {
            icon_name: "rating".repeat(16),
            count: 3,
            maximum: 5,
            color: Some("rating-color".repeat(16)),
            show_value: true,
        }),
    }
}

fn border_side(label: &str) -> SpreadsheetBorderSideArtifact {
    SpreadsheetBorderSideArtifact {
        style: label.repeat(16),
        color: Some(format!("{label}-color").repeat(16)),
    }
}

fn basic_cell(row: usize, text: &str) -> SpreadsheetCellArtifact {
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
