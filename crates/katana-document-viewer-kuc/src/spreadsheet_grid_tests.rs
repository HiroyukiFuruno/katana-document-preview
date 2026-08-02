use super::{
    KucSpreadsheetGridAdapter,
    mapping::{font_size, ratio_basis_points, track_size},
    test_support::{sample_cell, sample_sheet},
};
use katana_document_viewer::{SpreadsheetCoordinate, SpreadsheetMergedCellArtifact};
use katana_ui_core::molecule::{
    GridAction, GridCoordinate, GridEvent, GridHorizontalAlignment, GridNavigationIntent,
    GridVerticalAlignment, GridViewport,
};
use katana_ui_core::render_model::{UiGridValidationError, UiNode, UiNodeKind};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn large_sheet_requests_only_the_kuc_visible_window_and_maps_cells() -> TestResult {
    let mut adapter =
        KucSpreadsheetGridAdapter::new(&sample_sheet(), GridViewport::new(360, 140).scroll(80, 0))?;
    assert_eq!(3, adapter.sheet_index());
    let request = adapter.materialization_request();
    assert!(request.len() < 100);
    assert!(request.contains(&SpreadsheetCoordinate::new(0, 0)));
    assert!(request.contains(&SpreadsheetCoordinate::new(2, 2)));
    assert!(!request.contains(&SpreadsheetCoordinate::new(2, 3)));

    adapter.supply_cells(vec![sample_cell(SpreadsheetCoordinate::new(2, 2))])?;
    assert_materialized_cell(&adapter.node())?;
    Ok(())
}

fn assert_materialized_cell(node: &UiNode) -> TestResult {
    assert_eq!(UiNodeKind::Grid, node.kind());
    assert_eq!(
        (1_000, 100),
        (node.props().grid.row_count, node.props().grid.column_count)
    );
    let Some(cell) = node
        .props()
        .grid
        .cells
        .iter()
        .find(|cell| cell.coordinate == GridCoordinate::new(2, 2))
    else {
        return Err("materialized cell is missing from the KUC grid node".into());
    };
    assert_eq!("42.0", cell.text);
    assert_eq!((1, 2), (cell.row_span, cell.column_span));
    assert_materialized_appearance(cell);
    Ok(())
}

fn assert_materialized_appearance(cell: &katana_ui_core::render_model::UiGridCell) {
    assert_eq!(12, cell.appearance.font_size_px);
    assert_eq!(
        GridHorizontalAlignment::Center,
        cell.appearance.horizontal_alignment
    );
    assert_eq!(
        GridVerticalAlignment::Center,
        cell.appearance.vertical_alignment
    );
    assert_eq!(
        Some(6_250),
        cell.appearance
            .data_bar
            .as_ref()
            .map(|bar| bar.fill_ratio_basis_points)
    );
    assert_eq!(
        Some("arrow-up"),
        cell.appearance.icon.as_ref().map(|icon| icon.name.as_str())
    );
    assert_eq!(
        Some(4),
        cell.appearance.rating.as_ref().map(|rating| rating.count)
    );
}

#[test]
fn grid_actions_preserve_kuc_selection_and_scroll_contracts() -> TestResult {
    let mut adapter = KucSpreadsheetGridAdapter::new(&sample_sheet(), GridViewport::new(320, 120))?;
    assert_eq!(
        Some(GridCoordinate::new(0, 0)),
        adapter.grid().active_coordinate()
    );
    assert!(matches!(
        adapter.apply_action(GridAction::Navigate {
            intent: GridNavigationIntent::Down,
            extend: false,
        }),
        GridEvent::SelectionChanged(_)
    ));
    assert_eq!(
        Some(GridCoordinate::new(2, 0)),
        adapter.grid().active_coordinate()
    );
    assert!(matches!(
        adapter.apply_action(GridAction::ScrollTo { x: 96, y: 240 }),
        GridEvent::Scrolled(_)
    ));
    let baseline = KucSpreadsheetGridAdapter::new(&sample_sheet(), GridViewport::new(320, 120))?
        .materialization_request();
    assert_ne!(adapter.materialization_request(), baseline);
    Ok(())
}

#[test]
fn invalid_merged_cells_and_unrequested_cells_remain_typed_errors() -> TestResult {
    let mut invalid = sample_sheet();
    invalid.merged_cells = vec![SpreadsheetMergedCellArtifact {
        anchor: SpreadsheetCoordinate::new(0, 0),
        row_span: 1,
        column_span: 2,
    }];
    assert!(matches!(
        KucSpreadsheetGridAdapter::new(&invalid, GridViewport::new(320, 120)),
        Err(super::KucSpreadsheetGridError::InvalidGrid(
            UiGridValidationError::CellSpanCrossesFrozenBoundary { .. }
        ))
    ));

    let mut adapter = KucSpreadsheetGridAdapter::new(&sample_sheet(), GridViewport::new(100, 40))?;
    assert!(matches!(
        adapter.supply_cells(vec![sample_cell(SpreadsheetCoordinate::new(999, 99))]),
        Err(super::KucSpreadsheetGridError::InvalidGrid(
            UiGridValidationError::CellOutsideMaterializedRange { .. }
        ))
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
    let adapter = KucSpreadsheetGridAdapter::new(&empty, GridViewport::new(100, 100))?;
    assert_eq!(None, adapter.grid().active_coordinate());
    assert!(adapter.materialization_request().is_empty());

    assert_eq!(1, track_size(f32::NAN));
    assert_eq!(1, track_size(-1.0));
    assert_eq!(u32::MAX, track_size(f32::MAX));
    assert_eq!(0, font_size(f32::INFINITY));
    assert_eq!(0, font_size(0.0));
    assert_eq!(u16::MAX, font_size(f32::MAX));
    assert_eq!(0, ratio_basis_points(f64::NAN));
    assert_eq!(0, ratio_basis_points(-0.5));
    assert_eq!(10_000, ratio_basis_points(1.5));
    Ok(())
}
