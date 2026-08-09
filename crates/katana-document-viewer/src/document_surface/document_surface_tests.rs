use super::*;
use crate::{PdfRenderedPage, ViewerImageSurface};
use katana_ui_core::render_model::{
    UiGridCell, UiGridCoordinate, UiGridProps, UiGridViewport, UiNode, UiNodeKind, UiRect,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn zero_sized_viewport_is_normalized_at_the_kdv_boundary() {
    assert_eq!(DocumentViewport::new(1, 1), DocumentViewport::new(0, 0));
}

#[test]
fn active_text_is_available_only_for_a_materialized_active_grid_cell() -> TestResult {
    let coordinate = UiGridCoordinate { row: 1, column: 2 };
    let cell = UiGridCell {
        coordinate,
        text: "active".to_owned(),
        ..Default::default()
    };
    let frame = grid_frame(UiGridProps {
        active_cell: Some(coordinate),
        cells: vec![cell],
        ..Default::default()
    })?;
    assert_eq!(Some("active"), frame.active_text());

    let no_active = grid_frame(UiGridProps::default())?;
    assert_eq!(None, no_active.active_text());
    let missing = grid_frame(UiGridProps {
        active_cell: Some(coordinate),
        ..Default::default()
    })?;
    assert_eq!(None, missing.active_text());

    let page = DocumentSurfaceFrame::from_rendered_page("Page", rendered_page())?;
    assert_eq!(None, page.active_text());
    Ok(())
}

fn grid_frame(props: UiGridProps) -> Result<DocumentSurfaceFrame, DocumentSurfaceError> {
    DocumentSurfaceFrame::from_node(UiNode::new(UiNodeKind::Grid, "Grid").grid(props))
}

#[test]
fn neutral_grid_frame_keeps_backend_independent_geometry() -> TestResult {
    let coordinate = UiGridCoordinate { row: 2, column: 3 };
    let frame = grid_frame(neutral_grid_props(coordinate))?;
    let Some(grid) = frame.grid() else {
        return Err("grid frame is missing".into());
    };

    assert!(frame.page().is_none());
    assert_grid_geometry(grid);
    Ok(())
}

fn neutral_grid_props(coordinate: UiGridCoordinate) -> UiGridProps {
    UiGridProps {
        row_count: 20,
        column_count: 10,
        total_width: 800,
        total_height: 400,
        viewport: UiGridViewport::new(320, 160).scroll(40, 20),
        active_cell: Some(coordinate),
        show_grid_lines: false,
        cells: vec![UiGridCell {
            coordinate,
            bounds: UiRect::new(10, 20, 80, 40),
            clipped_bounds: UiRect::new(12, 22, 76, 36),
            text: "cell".to_owned(),
            row_span: 2,
            column_span: 3,
            frozen_row: true,
            frozen_column: true,
            accessibility_row_index: 3,
            accessibility_column_index: 4,
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn assert_grid_geometry(grid: &DocumentGridSurfaceFrame) {
    assert_eq!((20, 10), (grid.row_count, grid.column_count));
    assert_eq!((800, 400), (grid.total_width, grid.total_height));
    assert_eq!((320, 160), (grid.viewport.width, grid.viewport.height));
    assert_eq!((40, 20), (grid.scroll_x(), grid.scroll_y()));
    assert!(!grid.show_grid_lines);
    let cell = &grid.cells[0];
    assert_eq!((2, 3), (cell.row_span, cell.column_span));
    assert!(cell.frozen_row && cell.frozen_column);
    assert_eq!(
        (3, 4),
        (
            cell.accessibility_row_index,
            cell.accessibility_column_index
        )
    );
}

fn rendered_page() -> PdfRenderedPage {
    PdfRenderedPage {
        page_index: 0,
        scale: 1.0,
        surface: ViewerImageSurface {
            fingerprint: "page".to_owned(),
            width: 1,
            height: 1,
            display_width: 1.0,
            display_height: 1.0,
            content_scale: 100,
            rgba: vec![0, 0, 0, 255],
        },
    }
}

#[test]
fn unsupported_internal_surface_kind_is_a_typed_error() {
    assert!(matches!(
        DocumentSurfaceFrame::from_node(UiNode::new(UiNodeKind::Text, "text")),
        Err(DocumentSurfaceError::UnsupportedNodeKind { .. })
    ));
}
