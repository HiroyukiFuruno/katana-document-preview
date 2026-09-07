use super::{SpreadsheetGridSurface, test_support::sample_sheet};
use crate::{
    DocumentGridCommand, DocumentViewport, SpreadsheetAutoFilterArtifact, SpreadsheetCoordinate,
    SpreadsheetFilterRange,
};
use katana_ui_core::molecule::GridCoordinate;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn filter_visibility_preserves_grid_state_and_source_rows() -> TestResult {
    let mut sheet = sheet_with_filter();
    let viewport = DocumentViewport::new(320, 120);
    let mut surface = SpreadsheetGridSurface::new(&sheet, viewport)?;
    select_and_scroll(&mut surface);

    sheet
        .auto_filter
        .as_mut()
        .ok_or("filter missing")?
        .filtered_out_rows = vec![4];
    surface.replace_sheet(&sheet, viewport)?;

    assert_eq!(
        Some(GridCoordinate::new(5, 0)),
        surface.grid.active_coordinate()
    );
    let request = surface.materialization_request();
    assert!(!request.iter().any(|coordinate| coordinate.row == 1));
    assert!(!request.iter().any(|coordinate| coordinate.row == 4));
    let frame = surface.frame()?;
    assert!(frame.grid().ok_or("grid frame missing")?.viewport.scroll_y > 0);
    Ok(())
}

fn sheet_with_filter() -> crate::SpreadsheetSheetArtifact {
    let mut sheet = sample_sheet();
    sheet.auto_filter = Some(SpreadsheetAutoFilterArtifact {
        range: SpreadsheetFilterRange {
            start: SpreadsheetCoordinate::new(0, 0),
            end: SpreadsheetCoordinate::new(999, 9),
        },
        columns: Vec::new(),
        filtered_out_rows: Vec::new(),
        diagnostics: Vec::new(),
    });
    sheet
}

fn select_and_scroll(surface: &mut SpreadsheetGridSurface) {
    let _ = surface.apply_command(DocumentGridCommand::Select {
        row: 4,
        column: 0,
        extend: false,
    });
    let _ = surface.apply_command(DocumentGridCommand::ScrollTo { x: 0, y: 240 });
}
