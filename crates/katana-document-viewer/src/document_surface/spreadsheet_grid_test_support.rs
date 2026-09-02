use crate::{
    SpreadsheetBorderSideArtifact, SpreadsheetCellArtifact, SpreadsheetCellBorderArtifact,
    SpreadsheetCellStyleArtifact, SpreadsheetCellValue, SpreadsheetConditionalFormattingArtifact,
    SpreadsheetCoordinate, SpreadsheetDataBarArtifact, SpreadsheetHorizontalAlignment,
    SpreadsheetIconArtifact, SpreadsheetMergedCellArtifact, SpreadsheetRatingArtifact,
    SpreadsheetSheetArtifact, SpreadsheetTrackArtifact, SpreadsheetVerticalAlignment,
};

const TEXT_COLOR: [u8; 3] = [0x10, 0x20, 0x30];
const FILL_COLOR: [u8; 3] = [0xF0, 0xF1, 0xF2];
const ICON_COLOR: [u8; 3] = [0x00, 0x88, 0x00];
const RATING_COLOR: [u8; 3] = [0xFF, 0xCC, 0x00];
const POSITIVE_COLOR: [u8; 3] = [0x00, 0xAA, 0x00];
const NEGATIVE_COLOR: [u8; 3] = [0xAA, 0x00, 0x00];

pub(super) fn sample_sheet() -> SpreadsheetSheetArtifact {
    SpreadsheetSheetArtifact {
        index: 3,
        name: "Quarterly".to_owned(),
        row_count: 1_000,
        column_count: 100,
        row_tracks: sample_row_tracks(),
        column_tracks: vec![SpreadsheetTrackArtifact {
            size: 96.4,
            hidden: false,
        }],
        frozen_rows: 1,
        frozen_columns: 1,
        merged_cells: vec![SpreadsheetMergedCellArtifact {
            anchor: SpreadsheetCoordinate::new(2, 2),
            row_span: 1,
            column_span: 2,
        }],
        show_grid_lines: true,
        auto_filter: None,
    }
}

fn sample_row_tracks() -> Vec<SpreadsheetTrackArtifact> {
    vec![
        SpreadsheetTrackArtifact {
            size: 24.0,
            hidden: false,
        },
        SpreadsheetTrackArtifact {
            size: 0.0,
            hidden: true,
        },
    ]
}

pub(super) fn sample_cell(coordinate: SpreadsheetCoordinate) -> SpreadsheetCellArtifact {
    SpreadsheetCellArtifact {
        coordinate,
        display_text: "42.0".to_owned(),
        value: SpreadsheetCellValue::Number(42.0),
        formula: Some("=SUM(A1:A2)".to_owned()),
        style: sample_style(),
        conditional_formatting: sample_conditional_formatting(),
    }
}

fn sample_style() -> SpreadsheetCellStyleArtifact {
    SpreadsheetCellStyleArtifact {
        font_name: "Aptos".to_owned(),
        font_size: 11.6,
        font_color: Some(color(TEXT_COLOR)),
        fill_color: Some(color(FILL_COLOR)),
        bold: true,
        italic: true,
        underline: true,
        strike: true,
        horizontal_alignment: SpreadsheetHorizontalAlignment::CenterContinuous,
        vertical_alignment: SpreadsheetVerticalAlignment::Center,
        wrap_text: true,
        number_format: "0.0".to_owned(),
        borders: SpreadsheetCellBorderArtifact {
            left: Some(SpreadsheetBorderSideArtifact {
                style: "thin".to_owned(),
                color: Some(color([0xB7, 0xC4, 0xCE])),
            }),
            ..SpreadsheetCellBorderArtifact::default()
        },
    }
}

fn sample_conditional_formatting() -> SpreadsheetConditionalFormattingArtifact {
    SpreadsheetConditionalFormattingArtifact {
        applied: true,
        data_bar: Some(sample_data_bar()),
        icon: Some(SpreadsheetIconArtifact {
            name: "arrow-up".to_owned(),
            color: Some(color(ICON_COLOR)),
            show_value: true,
        }),
        rating: Some(SpreadsheetRatingArtifact {
            icon_name: "star".to_owned(),
            count: 4,
            maximum: 5,
            color: Some(color(RATING_COLOR)),
            show_value: false,
        }),
    }
}

fn sample_data_bar() -> SpreadsheetDataBarArtifact {
    SpreadsheetDataBarArtifact {
        positive_color: Some(color(POSITIVE_COLOR)),
        negative_color: Some(color(NEGATIVE_COLOR)),
        value: 0.625,
        axis_position: 0.25,
        gradient: true,
        show_value: false,
    }
}

fn color([red, green, blue]: [u8; 3]) -> String {
    format!("#{red:02X}{green:02X}{blue:02X}")
}
