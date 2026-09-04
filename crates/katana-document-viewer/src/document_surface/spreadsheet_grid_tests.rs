use super::{
    SpreadsheetGridSurface,
    appearance_tests::assert_materialized_appearance,
    mapping::{font_size, track_size},
    test_support::{sample_cell, sample_sheet},
};
use crate::{
    DocumentGridCommand, DocumentGridCoordinate, DocumentGridEvent, DocumentGridNavigation,
    DocumentGridSurfaceFrame, DocumentSurfaceError, DocumentViewport, SpreadsheetCoordinate,
    SpreadsheetMergedCellArtifact,
};
use katana_ui_core::molecule::GridCoordinate;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn large_sheet_requests_only_the_visible_window_and_maps_cells() -> TestResult {
    let mut surface =
        SpreadsheetGridSurface::new(&sample_sheet(), DocumentViewport::new(360, 140))?;
    let _ = surface.apply_command(DocumentGridCommand::ScrollTo { x: 80, y: 0 });
    assert_eq!(3, surface.sheet_index());
    let frame = surface.frame()?;
    let Some(grid) = frame.grid() else {
        return Err("spreadsheet did not produce a grid frame".into());
    };
    assert!(grid.show_grid_lines);
    let request = surface.materialization_request();
    assert!(request.len() < 100);
    assert!(request.contains(&SpreadsheetCoordinate::new(0, 0)));
    assert!(request.contains(&SpreadsheetCoordinate::new(2, 2)));
    assert!(!request.contains(&SpreadsheetCoordinate::new(2, 3)));

    surface.supply_cells(vec![sample_cell(SpreadsheetCoordinate::new(2, 2))])?;
    let frame = surface.frame()?;
    let Some(grid) = frame.grid() else {
        return Err("spreadsheet did not produce a grid frame".into());
    };
    assert_materialized_cell(grid)?;
    Ok(())
}

#[test]
fn sheet_grid_line_visibility_reaches_document_surface() -> TestResult {
    let mut sheet = sample_sheet();
    sheet.show_grid_lines = false;

    let surface = SpreadsheetGridSurface::new(&sheet, DocumentViewport::new(320, 120))?;

    let frame = surface.frame()?;
    let Some(grid) = frame.grid() else {
        return Err("spreadsheet did not produce a grid frame".into());
    };
    assert!(!grid.show_grid_lines);
    Ok(())
}

fn assert_materialized_cell(grid: &DocumentGridSurfaceFrame) -> TestResult {
    assert_eq!((1_000, 100), (grid.row_count, grid.column_count));
    let Some(cell) = grid
        .cells
        .iter()
        .find(|cell| cell.coordinate == DocumentGridCoordinate { row: 2, column: 2 })
    else {
        return Err("materialized cell is missing from the document grid".into());
    };
    assert_eq!("42.0", cell.text);
    assert_eq!((1, 2), (cell.row_span, cell.column_span));
    assert_materialized_appearance(cell);
    Ok(())
}

#[test]
fn grid_commands_preserve_selection_and_scroll_contracts() -> TestResult {
    let mut surface =
        SpreadsheetGridSurface::new(&sample_sheet(), DocumentViewport::new(320, 120))?;
    assert_initial_scroll_is_neutral(&mut surface);
    assert_eq!(
        Some(GridCoordinate::new(0, 0)),
        surface.grid.active_coordinate()
    );
    assert!(matches!(
        surface.apply_command(DocumentGridCommand::Navigate {
            intent: DocumentGridNavigation::Down,
            extend: false,
        }),
        DocumentGridEvent::SelectionChanged
    ));
    assert_eq!(
        Some(GridCoordinate::new(2, 0)),
        surface.grid.active_coordinate()
    );
    assert!(matches!(
        surface.apply_command(DocumentGridCommand::ScrollTo { x: 96, y: 240 }),
        DocumentGridEvent::Scrolled
    ));
    let baseline = SpreadsheetGridSurface::new(&sample_sheet(), DocumentViewport::new(320, 120))?
        .materialization_request();
    assert_ne!(surface.materialization_request(), baseline);
    Ok(())
}

fn assert_initial_scroll_is_neutral(surface: &mut SpreadsheetGridSurface) {
    assert_eq!(
        DocumentGridEvent::None,
        surface.apply_command(DocumentGridCommand::ScrollTo { x: 0, y: 0 })
    );
}

#[test]
fn merged_cells_adjust_frozen_boundaries_and_unrequested_cells_remain_typed_errors() -> TestResult {
    let mut invalid = sample_sheet();
    invalid.merged_cells = vec![SpreadsheetMergedCellArtifact {
        anchor: SpreadsheetCoordinate::new(0, 0),
        row_span: 2,
        column_span: 2,
    }];
    assert!(SpreadsheetGridSurface::new(&invalid, DocumentViewport::new(320, 120)).is_ok());

    let mut surface = SpreadsheetGridSurface::new(&sample_sheet(), DocumentViewport::new(100, 40))?;
    assert!(matches!(
        surface.supply_cells(vec![sample_cell(SpreadsheetCoordinate::new(999, 99))]),
        Err(DocumentSurfaceError::InvalidGrid { detail })
            if detail.contains("CellOutsideMaterializedRange")
    ));
    Ok(())
}

#[test]
fn empty_sheet_and_numeric_edge_cases_have_bounded_neutral_defaults() -> TestResult {
    let mut empty = sample_sheet();
    empty.row_count = 0;
    empty.column_count = 0;
    empty.row_tracks.clear();
    empty.column_tracks.clear();
    empty.frozen_rows = 0;
    empty.frozen_columns = 0;
    empty.merged_cells.clear();
    let surface = SpreadsheetGridSurface::new(&empty, DocumentViewport::new(100, 100))?;
    assert_eq!(None, surface.grid.active_coordinate());
    assert!(surface.materialization_request().is_empty());

    assert_eq!(1, track_size(f32::NAN));
    assert_eq!(1, track_size(-1.0));
    assert_eq!(u32::MAX, track_size(f32::MAX));
    assert_eq!(0, font_size(f32::INFINITY));
    assert_eq!(0, font_size(0.0));
    assert_eq!(u16::MAX, font_size(f32::MAX));
    Ok(())
}
