use super::{mapping::cell_content, test_support::sample_cell};
use crate::{SpreadsheetBorderSideArtifact, SpreadsheetCoordinate};
use katana_ui_core::render_model::UiGridBorderLineStyle;

fn color(bytes: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", bytes[0], bytes[1], bytes[2])
}

#[test]
fn spreadsheet_border_sides_project_to_public_kuc_grid_model() {
    let mut cell = sample_cell(SpreadsheetCoordinate::new(2, 2));
    cell.style.borders.right = Some(SpreadsheetBorderSideArtifact {
        style: "double".to_owned(),
        color: Some(color([0x11, 0x33, 0x55])),
    });
    cell.style.borders.top = Some(SpreadsheetBorderSideArtifact {
        style: "dotted".to_owned(),
        color: Some(color([0xAA, 0x55, 0x00])),
    });
    cell.style.borders.bottom = Some(SpreadsheetBorderSideArtifact {
        style: "future-visible-style".to_owned(),
        color: Some(color([0x88, 0x44, 0x00])),
    });

    let content = cell_content(cell);
    let borders = content.appearance.borders;

    assert_eq!(UiGridBorderLineStyle::Thin, borders.left.line_style);
    assert_eq!(Some("#B7C4CE"), borders.left.color.as_deref());
    assert_eq!(UiGridBorderLineStyle::Double, borders.right.line_style);
    assert_eq!(Some("#113355"), borders.right.color.as_deref());
    assert_eq!(UiGridBorderLineStyle::Dotted, borders.top.line_style);
    assert_eq!(Some("#AA5500"), borders.top.color.as_deref());
    assert_eq!(UiGridBorderLineStyle::Solid, borders.bottom.line_style);
    assert_eq!(Some("#884400"), borders.bottom.color.as_deref());
}
